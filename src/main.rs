use fancy_regex::Regex;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::{params, Connection, Result};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
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

fn capitalize_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn parse_and_write_db(contents: &str, db_conn: Arc<Mutex<Connection>>, lang_map: HashMap<String, String>) -> Result<()> {
    // HashMap to store stuff in memory until written to database
    let mut pages_to_links: HashMap<String, HashSet<String>> = HashMap::new();

    // Regex pattern to find links to other wikipedia pages
    let links_regex = Regex::new(
        r"(?<internal>(?<=\[\[)(?!File:)(?!Category:)[\w\(\) -]*(?=|\]\]))|(?<lang>(?<={{etymology\|)[a-z]{1,3})",
    )
    .unwrap();

    // xml reader object
    let mut reader = Reader::from_str(&contents);
    let mut cur_page = String::default();
    let mut cur_state: State = State::IDLE;
    let mut count: u64 = 0;
    let start = Instant::now();
    loop {
        match reader.read_event() {
            /*  This will only happen in the case that a thread reads the closing </mediawiki> tag without having read 
            the opening tag. This is fine because we know we will have read that opening tag in the thread allocated to the
            first part of the file, so this will indicate to use that the thread is done reading its portion of the file */
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
            State::IGNORE is set whenever encountering a self-closing redirect tag */
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

            /* This event handles all text within the dump file. We only really want to handle text if the cur_state is 
            State::TITLE (Meaning we just saw a title tag) or State::TEXT (We've hit a text tag and our cur_state is not
            State::IGNORE). In the former, the text that we read will be the name of the page, in the latter, the text will
            be the actual content on that page. When reading the content on the page, we use the links_regex to capture all links 
            to other wikipedia articles. Those links will appear as text surrounded by [[ ]], so the link to Canada will be [[Canada]].
            Links might also be a part of a sentence and so might not be exactly the name, something like [[canada|the country of canada]]
            where the stuff after the | is the text, in that case we know that the text before the bar is the title.
            
            We'll consider valid wikipedia links to be one of two types. The first is just a regular page link. The second are links to languages
            like Latin or Arabic in the case that the text links to those pages when explaining a words etymology. For example on the page
            for Albedo we see (/ælˈbiːdoʊ/ al-BEE-doh; from Latin albedo 'whiteness'). In that case we consider Latin to be a valid link.
            When matching a language link, it will be appear in the text as an iso 639 code which we'll need to use to determine the language it's referencing.
            For example Latin has the iso 639 code 'la' so in text it will show up as 'la' not 'Latin' (There are cases where 'Latin' is a link but that's 
            handled in the first case).
            */
            Ok(Event::Text(e)) => match cur_state {
                State::TITLE => {
                    cur_page = String::from(e.unescape().unwrap().into_owned());
                    pages_to_links.insert(cur_page.clone(), HashSet::new());
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
                                        .insert(String::from(capitalize_first_char(val.as_str()))),
                                    // Some(val) => println!("Internal: {}", val.as_str()),
                                    // Some(val) => (),
                                    None => false,
                                };
                                match cap.name("lang") {
                                    Some(val) => {
                                        let lang_code = val.as_str();
                                        if let Some(lang) = lang_map.get(lang_code) {
                                            pages_to_links
                                                .get_mut(&cur_page)
                                                .unwrap()
                                                .insert(lang.clone());
                                        }
                                        true
                                    },
                                    // Some(val) => println!("Lang: {}", val.as_str()),
                                    // Some(val) => (),
                                    None => false,
                                };
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

    /* Once the thread has finished processing its section, it tries to obtain the db connection mutex to start inserting 
    data from pages_to_links, this is better than having threads try to obtain the mutex while processing its section */
    let connection = db_conn.lock().unwrap();

    // Prepared statements to insert a page title into the PAGES table and get the id from the page after its inserted
    let mut page_title_insert = connection.prepare("insert into PAGES(page_title) values(?1);").unwrap();
    let mut get_last_id = connection.prepare("select id from PAGES where page_title = (?1);").unwrap();
    
    for entry in pages_to_links {
        let page_title = entry.0;
        let links = entry.1;
        /*  This string will be used to execute a batch insert for all links corresponding to an entry, better than executing an
        insert for every single links */
        let mut insert_links = String::from("BEGIN;\n"); 

        // Insert the current page title, get its id in the pages database
        page_title_insert.execute(params![page_title])?;
        let last_id: i64 = get_last_id.query_row(params![page_title], |row| row.get(0))?;

        // Start building the string for the insert statement then execute
        for link in links {
            insert_links.push_str(&format!("insert into LINKS(page_id, link_title) values({}, '{}');\n", last_id, link).to_string());
        }
        insert_links.push_str("COMMIT;");
        let _ = connection.execute_batch(&insert_links);
    }
    // count excluding redirects: 21171
    // Time taken 1132.105713876s
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
    let total_time_start = Instant::now();
    let path = "enwiki-latest-pages-articles-multistream1.xml-p1p41242";
    let contents_file = File::open(&path)
        .expect("Can't find file");
    let num_cpus = available_parallelism().unwrap().get();
    // let initial_connection = Connection::open("test.db").unwrap();

    let conn = Connection::open("test.db").unwrap();
    let conn_ref = &conn;

    let mut lang_map: HashMap<String, String> = HashMap::new();
    let mut stmt = conn_ref.prepare("select * from LANGUAGE_CODES").unwrap();
    let rows = stmt.query_map([], |row| {
        let col0: String = row.get(0)?;
        let col1: String = row.get(1)?;
        Ok(vec![col0, col1])
    }).unwrap();
    for r in rows {
        let temp = r.unwrap();
        lang_map.insert(temp[0].clone(), temp[1].clone());
    }
    drop (stmt);

    let conn = Arc::new(Mutex::new(conn));
    
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
        let lang_map_clone = lang_map.clone();
        let handle = thread::spawn(move || parse_and_write_db(&group.as_str(), conn_clone, lang_map_clone));
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.join().expect("Thread panicked!");
    }
    let total_time_end = total_time_start.elapsed();
    println!("Processing of {} took {:?}", path, total_time_end);
}
