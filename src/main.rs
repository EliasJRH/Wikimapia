use fancy_regex::Regex;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::thread::{self, available_parallelism};
use std::time::Instant;

#[derive(Debug)]
enum State {
    IDLE,
    TITLE,
    IGNORE,
    TEXT,
}

fn parse_and_write_db(contents: &str, db_conn: Arc<Mutex<Connection>>) -> Result<()> {
    let mut pages_to_links: HashMap<String, Vec<String>> = HashMap::new();
    let links_regex = Regex::new(
        r"(?<internal>(?<=\[\[)(?!File:)(?!Category:)[\w\(\) -]*(?=|\]\]))|(?<lang>(?<={{etymology\|)[a-z]{1,3})",
    )
    .unwrap();
    let mut reader = Reader::from_str(&contents);
    let mut cur_page = String::default();
    let mut cur_state: State = State::IDLE;
    let mut count: u64 = 0;
    let start = Instant::now();
    // let insert_statement = String::from("BEGIN;");
    loop {
        match reader.read_event() {
            Err(e) => match e {
                quick_xml::Error::EndEventMismatch { .. } => break,
                _ => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            },
            // If EOF, break out of loop
            Ok(Event::Eof) => break,

            /* In order to tag to be valid, it must not contain an empty redirect tag.
            A self-closed redirect tag indicates that that revision just modified a link to redirect to another article
            We don't care about those. We want pages that don't contain a redirect tag, but because redirect tags always
            come before text tags that contain actual content, we need to check if a redirect came before. That's why
            State::IGNORE is set whenever encountering a self-closing redirect tag*/
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"title" => cur_state = State::TITLE,
                b"text" => match cur_state {
                    State::IGNORE => (),
                    _ => cur_state = State::TEXT,
                },
                _ => (),
            },
            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"redirect" => {
                    cur_state = State::IGNORE;
                    pages_to_links.remove(&cur_page);
                }
                _ => (),
            },

            /* If we hit text while still being in the State::TEXT state, then we'll know that the page is not a
            redirect and we should parse the content associated with the */
            Ok(Event::Text(e)) => match cur_state {
                State::TITLE => {
                    cur_page = String::from(e.unescape().unwrap().into_owned());
                    pages_to_links.insert(cur_page.clone(), Vec::new());
                    cur_state = State::IDLE;
                }
                State::TEXT => {
                    count += 1;
                    // println!("count: {}", count);
                    let cur_text = String::from(e.unescape().unwrap().into_owned());
                    let mut captures = links_regex.captures_iter(&cur_text);
                    loop {
                        let first = captures.next();
                        if first.is_none() {
                            break;
                        }
                        match first.unwrap() {
                            Ok(cap) => {
                                match cap.name("internal") {
                                    Some(val) => pages_to_links
                                        .get_mut(&cur_page)
                                        .unwrap()
                                        .push(String::from(val.as_str())),
                                    // Some(val) => println!("Internal: {}", val.as_str()),
                                    // Some(val) => (),
                                    None => (),
                                }
                                match cap.name("lang") {
                                    Some(val) => pages_to_links
                                        .get_mut(&cur_page)
                                        .unwrap()
                                        .push(String::from(val.as_str())),
                                    // Some(val) => println!("Lang: {}", val.as_str()),
                                    // Some(val) => (),
                                    None => (),
                                }
                            }
                            Err(_c) => break,
                        }
                    }
                    cur_state = State::IDLE;
                }
                _ => (),
            },

            // There are several other `Event`s we do not consider here
            _ => (),
        }
    }
    let end = start.elapsed();
    println!("Time taken {:?}", end);
    println!("Number of articles processed: {}", count);
    let connection = db_conn.lock().unwrap();
    for entry in pages_to_links {
        let page_title = entry.0;
        let links = entry.1;
        connection.execute("insert into PAGES(page_title) values(?1);", params![page_title])?;
    }
    // count excluding redirects: 21171
    // Time taken 1132.105713876ss
    // pages_to_links
    Ok(())
}

fn divide_input(contents_file: File, divisions: Option<usize>) -> Vec<String> {
    // Create BufferedReader to determine size of file
    let mut file_reader = BufReader::new(contents_file);
    let divisions = divisions.unwrap_or(12);

    let total_line_count = (&mut file_reader).lines().count();
    println!("Total line count: {}", total_line_count);

    /* Move BufferedReader back to beginning of file to start dividing file into mostly equal
    parts.
    */
    let _ = file_reader.seek(SeekFrom::Start(0));
    let mut content_vec: Vec<String> = Vec::new(); // Vector containing each part
    let mut cur_line_count = 0;

    /* A page must be fully contained within each block, we can't have part of a page be
    in one block and the rest be in another as that would mess up parsing. Therefore, we
    read at least total_line_count/divisions lines then check if the block ends with </page>
    indicating the end of the page. If not, we just keep adding to the block until it does*/
    for i in 1..divisions {
        cur_line_count = 0;
        let mut section = String::new();
        for _ in 0..(total_line_count / divisions) {
            let _ = file_reader.read_line(&mut section);
            cur_line_count += 1;
        }

        if section.ends_with("</page>") {
            continue;
        }
        loop {
            let _ = file_reader.read_line(&mut section);
            cur_line_count += 1;
            if section.ends_with("</page>\n") {
                break;
            }
        }
        assert!(section.ends_with("</page>\n"));
        println!("Section {} line count: {}", i, cur_line_count);
        content_vec.push(section);
        cur_line_count = 0;
    }

    /* The last section must contain the rest of the file, so we read until EOF */
    let mut last_section = String::new();
    loop {
        let bytes = file_reader.read_line(&mut last_section);
        match bytes {
            Ok(c) => {
                if c == 0 {
                    break;
                } else {
                    cur_line_count += 1;
                }
            }
            _ => (),
        }
    }
    assert!(last_section.ends_with("</mediawiki>"));
    println!("Section {} line count: {}", divisions, cur_line_count);

    content_vec.push(last_section);
    content_vec
}

fn main() {
    let contents_file = File::open("enwiki-latest-pages-articles-multistream1.xml-p1p41242")
        .expect("Can't find file");
    let num_cpus = available_parallelism().unwrap().get();
    // let initial_connection = Connection::open("test.db").unwrap();
    let conn = Arc::new(Mutex::new(Connection::open("test.db").unwrap()));

    // 1 division -> 18 minutes
    // 6 divisions -> 8 minutes
    // 12 divisions -> 5.45 minutes
    // 32 divisions -> Total time: 6.26 minutes
    // 64 divisions -> Total time: 5.86 minutes
    // Pretty much once you spawn num_cpu threads, performance doesn't increase
    let content_vec = divide_input(contents_file, Some(num_cpus));

    let mut handles: Vec<thread::JoinHandle<Result<(), rusqlite::Error>>> = vec![];
    for group in content_vec {
        let conn_clone = Arc::clone(&conn);
        let handle = thread::spawn(move || parse_and_write_db(&group.as_str(), conn_clone));
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join().expect("Thread panicked!");
    }
}
