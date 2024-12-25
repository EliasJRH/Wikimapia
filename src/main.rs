use fancy_regex::Regex;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::time::Instant;

#[derive(Debug)]
enum State {
    IDLE,
    TITLE,
    IGNORE,
    TEXT,
}

fn parse_contents(contents: &str) -> HashMap<String, Vec<String>> {
    let mut pages_to_links: HashMap<String, Vec<String>> = HashMap::new();
    let links_regex = Regex::new(
        r"(?<internal>(?<=\[\[)(?!File:)(?!Category:)[\w\(\) -]*(?=|\]\]))|(?<lang>(?<={{etymology\|)[a-z]{1,3})",
    )
    .unwrap();
    let mut reader = Reader::from_str(&contents);
    let mut cur_page = String::default();
    let mut cur_text = String::default();
    let mut cur_state: State = State::IDLE;
    let mut count: u64 = 0;
    let start = Instant::now();
    loop {
        // Idea, split file into 4 sections of roughly equal size, then have a thread compute each section
        // xml reader reads each line of xml input
        match reader.read_event() {
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            // If EOF, break out of loop
            Ok(Event::Eof) => break,

            /* In order to tag to be valid, it must not contain an empty redirect tag.
            A self-closed redirect tag indicated that that revision just modified a link to redirect to another article
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
                b"redirect" => cur_state = State::IGNORE,
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
                    println!("count: {}", count);
                    cur_text = String::from(e.unescape().unwrap().into_owned().as_str());
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
                            Err(c) => break,
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
    // println!("{:?}", pages_to_links);
    // count including redirects: 27371
    // count excluding redirects: 21174
    // Time taken 1132.105713876ss
    return pages_to_links;
}

fn main() {
    //
    let contents_file = File::open("enwiki-latest-pages-articles-multistream1.xml-p1p41242")
        .expect("Can't find file");
    let mut file_reader = BufReader::new(contents_file);
    let line_count = (&mut file_reader).lines().count();
    println!("{}", line_count);
    let _ = file_reader.seek(SeekFrom::Start(0));
    let mut buf = String::new();
    let _ = (&mut file_reader).read_line(&mut buf);
    let mut content_vec: Vec<&str> = Vec::with_capacity(4);
    for i in 1..4 {
        let mut section = String::new();
        for _ in 0..(line_count / 4) {
            let _ = file_reader.read_line(&mut section);
        }
        if section.ends_with("</page>"){ continue; }
        loop {
            let _ = file_reader.read_line(&mut section);
            if section.ends_with("</page>\n"){
                break;
            }
        }
        // content_vec.push(section.as_mut_str());
    }
    
    // println!("{}", buf);
    // let contents = fs::read_to_string("enwiki-latest-pages-articles-multistream1.xml-p1p41242")
    //     .expect("Should have been able to read");
    // parse_contents(&contents);
}
