use bzip2::read::BzDecoder;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use regex::{Regex, RegexBuilder};
use rusqlite::{params, Connection, Result};
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{remove_file, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, available_parallelism};
use std::time::{Duration, Instant};
use std::usize;

#[derive(Debug)]
enum State {
    IDLE,
    TITLE,
    IGNORE,
    TEXT,
    NAMESPACE,
}

fn capitalize_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn process_article_name<'a>(name: &'a str, _namespace_regex: &Regex) -> Option<&'a str> {
    if name.starts_with(":") {
        return None;
    }
    let mut split = name.split("|");
    if let Some(processed_name) = split.next() {
        if _namespace_regex.is_match(&(processed_name.split(" ").next().unwrap())) {
            return None;
        }
        return Some(&processed_name);
    }
    return None;
}

fn parse_and_write_db(
    thread_id: usize,
    contents: &str,
    db_conn: Arc<Mutex<Connection>>,
    lang_map: HashMap<String, String>,
) -> Result<()> {
    // HashMap to store stuff in memory until written to database
    let mut pages_to_links: HashMap<String, HashSet<String>> = HashMap::new();
    let mut redirects_to_pages: HashMap<String, String> = HashMap::new();

    // Regex to find internal wikipedia links and links to language pages
    // Internal wikipedia links look like [[text]], language links look like {{etymology|<language code>
    // The language code is looked up to find the name of the languages article
    let links_regex = RegexBuilder::new(
        r"(\[\[[A-Za-z0-9 .,:()'&+-/|{}=?\u0080-\uFFFF]+\]\])|(\{\{etymology\|[a-z]{1,3})",
    )
    .case_insensitive(true)
    .build()
    .unwrap();

    let namespace_regex = RegexBuilder::new(r"\w*:\S\w*")
        .case_insensitive(true)
        .build()
        .unwrap();

    // xml reader object
    let mut reader = Reader::from_str(&contents);
    let mut cur_page = String::default();
    let mut cur_state: State = State::IDLE;
    let mut count: usize = 0;
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
                b"ns" => cur_state = State::NAMESPACE,
                _ => (),
            },
            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"redirect" => {
                    cur_state = State::IGNORE;
                    pages_to_links.remove(&cur_page);
                    if let Some(attribute) = e.attributes().next() {
                        let redirect_title = String::from(
                            attribute
                                .unwrap()
                                .decode_and_unescape_value(&reader)
                                .unwrap(),
                        );
                        redirects_to_pages.insert(cur_page.clone(), redirect_title);
                    }
                }
                _ => (),
            },

            /* This event handles all text within the dump file. We only really want to handle text in a few cases.
            Either cur_state is State::TITLE (Meaning we just saw a title tag) or State::TEXT (We've hit a text tag and our cur_state is not
            State::IGNORE) or State::NAMESPACE (We just saw a namespace tag). In the former, the text that we read will
            be the name of the page, in the second case its the namespace id and in the latter, the text will be the actual content on that page. When reading the content on
            the page, we use the links_regex to capture all links to other wikipedia articles. Those links will appear as text surrounded
            by [[ ]], so the link to Canada will be [[Canada]]. Links might also be a part of a sentence and so might not be exactly the
            name, something like [[canada|the country of canada]] where the stuff after the | is the text, in that case we know that the text
            before the bar is the title.

            We'll consider valid wikipedia links to be one of two types. The first is just a regular page link. The second are links to languages
            like Latin or Arabic in the case that the text links to those pages when explaining a words etymology. For example on the page
            for Albedo we see (/ælˈbiːdoʊ/ al-BEE-doh; from Latin albedo 'whiteness'). In that case we consider Latin to be a valid link.
            When matching a language link, it will be appear in the text as an iso 639 code which we'll need to use to determine the language it's referencing.
            For example Latin has the iso 639 code 'la' so in text it will show up as 'la' not 'Latin' (There are cases where 'Latin' is a link but that's
            handled in the first case).

            Another thing to mention is Wikipedia namespaces. A namespace is an identifier for a wikipedia page that categorizes it as one of 28 types. One
            of these types are normal wikipedia articles but there are also pages for files, help, drafts, and others. We're only concerned with actual
            Wikipedia articles who namespace id is 0, everything else we'll ignore. If we see a namespace tag <ns>, cur_state is set to State::NAMESPACE to
            read the namespace id as text. If the namespace id is anything else but 0, we set cur_state to State::IGNORE similarly to how its done for redirects

            Namespaces also have their own internal link structure, so the link regex also checks to make sure that we're not capturing those as well
            */
            Ok(Event::Text(e)) => match cur_state {
                State::TITLE => {
                    cur_page = String::from(e.unescape().unwrap().into_owned());
                    pages_to_links.insert(cur_page.clone(), HashSet::new());
                    cur_state = State::IDLE;
                }
                State::NAMESPACE => {
                    let ns_num: i32 = String::from(e.unescape().unwrap().into_owned())
                        .parse()
                        .unwrap();
                    if ns_num != 0 {
                        cur_state = State::IGNORE;
                        pages_to_links.remove(&cur_page);
                    } else {
                        cur_state = State::IDLE;
                    }
                }
                State::TEXT => {
                    count += 1;
                    let cur_text = String::from(e.unescape().unwrap().into_owned());
                    let captures = links_regex.captures_iter(&cur_text);
                    for cap in captures {
                        if let Some(val) = cap.get(1) {
                            let name_slice = &val.as_str()[2..val.len() - 2];
                            if let Some(article_name) =
                                process_article_name(name_slice, &namespace_regex)
                            {
                                pages_to_links
                                    .get_mut(&cur_page)
                                    .unwrap()
                                    .insert(String::from(capitalize_first_char(article_name)));
                            }
                        }
                        if let Some(val) = cap.get(2) {
                            let lang_code = &val.as_str()[12..];
                            if let Some(lang_name) = lang_map.get(lang_code) {
                                pages_to_links
                                    .get_mut(&cur_page)
                                    .unwrap()
                                    .insert(lang_name.clone());
                            }
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
    assert_eq!(count, pages_to_links.len());
    let end = start.elapsed();
    println!("Time taken {:?}", end);
    println!("Number of articles processed: {}", count);

    /* Once the thread has finished processing its section, it tries to obtain the db connection mutex to start inserting
    data from pages_to_links, this is better than having threads try to obtain the mutex while processing its section */
    let mut connection = db_conn.lock().unwrap();

    for entry in pages_to_links {
        let page_title = entry.0;
        let links = entry.1;

        // Prepared statements to insert a page title into the PAGES table and get the id from the page after its inserted
        let mut page_title_insert = connection
            .prepare("insert into PAGES(page_title) values(?1);")
            .unwrap();
        let mut get_last_id = connection
            .prepare("select id from PAGES where page_title = (?1);")
            .unwrap();

        // Insert the current page title, get its id in the pages database
        page_title_insert.execute(params![page_title])?;
        let last_id: i64 = get_last_id.query_row(params![page_title], |row| row.get(0))?;

        drop(page_title_insert);
        drop(get_last_id);

        let insert_page_title_tx = connection.transaction().unwrap();

        let mut insert_page_stmt = (&insert_page_title_tx)
            .prepare("insert into LINKS(page_id, link_title) values(?1, ?2);")
            .unwrap();

        for link in links {
            let res = insert_page_stmt.execute(params![last_id, link]);
            match res {
                Ok(_) => (),
                Err(e) => eprintln!(
                    "Error inserting link {} for page {}: {}",
                    link, page_title, e
                ),
            }
        }
        drop(insert_page_stmt);

        let res = insert_page_title_tx.commit();
        match res {
            Ok(_) => (),
            Err(e) => eprintln!("Error inserting links for page {}: {}", page_title, e),
        }
    }
    let insert_redirects_tx = connection.transaction().unwrap();
    let mut insert_redirects_stmt = (&insert_redirects_tx)
        .prepare("insert into REDIRECTS(page_title, redirect_title) values (?1, ?2)")
        .unwrap();

    for redirect in redirects_to_pages {
        let page_title = redirect.0;
        let redirect_title = redirect.1;

        let res = insert_redirects_stmt.execute(params![page_title, redirect_title]);
        match res {
            Ok(_) => (),
            Err(e) => eprintln!(
                "Error inserting redirect {} for page {}: {}",
                redirect_title, page_title, e
            ),
        }
    }

    drop(insert_redirects_stmt);

    let res = insert_redirects_tx.commit();
    match res {
        Ok(_) => (),
        Err(e) => eprintln!("Error inserting redirects: {}", e),
    }

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

fn get_files() -> Result<VecDeque<String>, Box<dyn std::error::Error>> {
    let base_url = "https://dumps.wikimedia.org/enwiki/latest/";
    let prefix = "enwiki-latest-pages-articles";
    let suffix = ".bz2";

    // Step 1: Fetch the directory listing
    println!("Fetching directory listing from {}", base_url);
    let html = reqwest::blocking::get(base_url)?.text()?;

    // Step 2: Parse the HTML to find files with the given prefix
    println!("Parsing directory listing...");
    let document = Html::parse_document(&html);
    let selector = Selector::parse("a").unwrap();
    let mut files_to_download = VecDeque::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with(prefix)
                && href.ends_with(suffix)
                && !href.contains("multistream")
                && !href.contains("articles.xml")
            {
                files_to_download.push_front(href.to_string());
            }
        }
    }
    Ok(files_to_download)
}

fn download_decompress_save_to_file(file_name: &String) -> Result<File, std::io::Error> {
    let base_url = "https://dumps.wikimedia.org/enwiki/latest/";
    let file_url = format!("{}{}", base_url, file_name);
    let client = reqwest::blocking::Client::new();

    let start_download = Instant::now();
    println!("Downloading {}", file_name);
    let response = client
        .get(&file_url)
        .timeout(Duration::from_secs(600))
        .send()
        .unwrap();
    if !response.status().is_success() {
        eprintln!("Failed to download {}: {}", file_url, response.status());
    }
    let compressed_data = response.bytes().unwrap();
    let end_download = start_download.elapsed();
    println!("{} downloaded in {:?}", file_name, end_download);

    println!("Decompressing {}", file_name);
    let cursor = std::io::Cursor::new(compressed_data);
    let mut decompressor = BzDecoder::new(cursor);

    let mut temp_file_path = std::env::temp_dir();
    temp_file_path.push("decompressed_file.tmp");
    let mut temp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp_file_path)
        .unwrap();

    let mut buffer = Vec::new();
    let _ = decompressor.read_to_end(&mut buffer);
    temp_file.write_all(&buffer).unwrap();
    println!("{} written to temp file", file_name);
    File::open(&temp_file_path)
}

fn seed_db() -> Result<()> {
    let total_time_start = Instant::now();
    let mut files_to_download = get_files().unwrap();
    let num_sections = files_to_download.len();
    println!("Directory listing contains {} items", num_sections);

    /* Some pre-initialization stuff, figure out how much cpus are available for
    multi-threading, store the language_codes table into memory so it can be used
    by threads */
    let mut sections_processed = 0;
    let num_cpus = available_parallelism().unwrap().get();
    let db_path = "main.db";

    let setup_connection = Connection::open(db_path).unwrap();
    let conn_ref = &setup_connection;

    let create_tables = std::fs::read_to_string("create_tables.sql").unwrap();
    let language_codes = std::fs::read_to_string("language_codes.sql").unwrap();
    conn_ref.execute_batch(&create_tables).unwrap();
    conn_ref.execute_batch(&language_codes).unwrap();

    let mut lang_map: HashMap<String, String> = HashMap::new();
    let mut stmt = conn_ref.prepare("select * from LANGUAGE_CODES").unwrap();
    let rows = stmt
        .query_map([], |row| {
            let col0: String = row.get(0)?;
            let col1: String = row.get(1)?;
            Ok(vec![col0, col1])
        })
        .unwrap();
    for r in rows {
        let temp = r.unwrap();
        lang_map.insert(temp[0].clone(), temp[1].clone());
    }

    for section in files_to_download {
        let section_time_start = Instant::now();
        let contents_file = download_decompress_save_to_file(&section).unwrap();

        let connection = Connection::open(db_path).unwrap();
        let _ = connection.execute("PRAGMA synchronous = OFF;", params![]);
        let conn_mutex = Arc::new(Mutex::new(connection));

        // 1 division -> 18 minutes
        // 6 divisions -> 8 minutes
        // 12 divisions -> 5.45 minutes
        // 32 divisions -> Total time: 6.26 minutes
        // 64 divisions -> Total time: 5.86 minutes
        // Pretty much once you spawn num_cpu threads, performance doesn't increase
        let content_vec = divide_input(contents_file, Some(num_cpus));

        let mut handles: Vec<thread::JoinHandle<Result<(), rusqlite::Error>>> = vec![];
        for (i, group) in content_vec.into_iter().enumerate() {
            let conn_clone = Arc::clone(&conn_mutex);
            let lang_map_clone = lang_map.clone();
            let handle = thread::spawn(move || {
                parse_and_write_db(i, &group.as_str(), conn_clone, lang_map_clone)
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join().expect("Thread panicked!");
        }

        let section_time_end = section_time_start.elapsed();
        sections_processed += 1;
        println!(
            "Processing of {} took {:?}. Sections processed: {}/{}",
            section, section_time_end, sections_processed, num_sections
        );
    }
    let _ = remove_file("/tmp/decompressed_file.tmp");
    let total_time_end = total_time_start.elapsed();
    let create_indexes = std::fs::read_to_string("create_indexes.sql").unwrap();
    conn_ref.execute_batch(&create_indexes).unwrap();
    println!(
        "Processing all Wikipedia sections took: {:?}",
        total_time_end
    );
    // Total time nearly 8 hours!
    // Total time only processing articles roughly 6 hours
    // Maybe I'm an idiot???
    // println!("Processing of {:?} took {:?}", path, article_time_end);
    Ok(())
}

fn check_for_page(page_name: &str) -> Result<String> {
    let check_conn = Connection::open("main.db").unwrap();
    check_conn.query_row(
        "select * from PAGES where page_title = (?1)",
        params![page_name],
        |row| row.get(1),
    )
}

fn find_redirect(page_name: &str) -> Result<String> {
    let find_redirect_conn = Connection::open("main.db").unwrap();
    find_redirect_conn.query_row(
        "select redirect_title from redirects where page_title = (?1)",
        params![page_name],
        |row| row.get(0),
    )
}

fn find_depth(start_page: &str) -> Result<()> {
    let search_start = Instant::now();
    let mut seen: HashMap<String, String> = HashMap::new();
    seen.insert(start_page.to_string(), start_page.to_string());
    let mut queue: VecDeque<(String, i32)> = VecDeque::from([(String::from(start_page), 0)]);
    let mut max_depth = 0;

    let search_conn = Connection::open("main.db").unwrap();
    let mut get_page_id = search_conn
        .prepare("select id from PAGES where page_title = (?1)")
        .unwrap();
    let mut find_links = search_conn
        .prepare("select link_title from LINKS where page_id = (?1)")
        .unwrap();

    while !queue.is_empty() {
        let cur = queue.pop_front().unwrap();
        let cur_name = cur.0;
        let cur_depth = cur.1;
        max_depth = std::cmp::max(max_depth, cur_depth);
        let cur_id: usize = get_page_id
            .query_row([cur_name.clone()], |row| row.get(0))
            .unwrap();
        let links = find_links
            .query_map([cur_id], |row| {
                let pt: String = row.get(0)?;
                Ok(pt)
            })
            .unwrap();
        for link in links {
            // println!("{}", link);
            let mut link_str = link?;
            // println!("{}: {}", cur, link_str);
            if let Err(e) = check_for_page(&link_str) {
                if let Ok(redirect) = find_redirect(&link_str) {
                    link_str = redirect;
                } else {
                    continue;
                }
            }
            if !seen.contains_key(&link_str) {
                seen.insert(link_str.clone(), cur_name.clone());
                queue.push_back((link_str.clone(), cur_depth + 1));
            }
        }
    }

    println!("Max depth: {}", max_depth);

    Ok(())
}

fn find_shortest_path(start_page: &str, end_page: &str) -> Result<()> {
    // seen maps node to parent
    // parent of start_page is start_page
    let search_start = Instant::now();
    let mut seen: HashMap<String, (String, Option<String>)> = HashMap::new();
    seen.insert(start_page.to_string(), (start_page.to_string(), None));
    let mut queue: VecDeque<String> = VecDeque::from([String::from(start_page)]);

    let search_conn = Connection::open("main.db").unwrap();
    let mut get_page_id = search_conn
        .prepare("select id from PAGES where page_title = (?1)")
        .unwrap();
    let mut find_links = search_conn
        .prepare("select link_title from LINKS where page_id = (?1)")
        .unwrap();

    while !queue.is_empty() {
        let mut found = false;
        let cur = queue.pop_front().unwrap();
        let mut cur_id: usize = 0;
        let res = get_page_id.query_row([cur.clone()], |row| row.get(0));
        let mut is_redirect = false;
        let mut redirect_str = String::new();
        match res {
            Ok(id) => cur_id = id,
            Err(e) => {
                eprintln!("Error getting page id for {}: {}", cur, e);
                continue;
            }
        }
        let links = find_links
            .query_map([cur_id], |row| {
                let pt: String = row.get(0)?;
                Ok(pt)
            })
            .unwrap();
        for link in links {
            // println!("{}", link);
            let mut link_str = link?;
            // println!("{}: {}", cur, link_str);
            if let Err(e) = check_for_page(&link_str) {
                if let Ok(redirect) = find_redirect(&link_str) {
                    redirect_str = link_str;
                    is_redirect = true;
                    link_str = redirect;
                } else {
                    continue;
                }
            }
            if !seen.contains_key(&link_str) {
                if is_redirect {
                    seen.insert(link_str.clone(), (cur.clone(), Some(redirect_str.clone())));
                }else {
                    seen.insert(link_str.clone(), (cur.clone(), None));
                }
                is_redirect = false;
                queue.push_back(link_str.clone());
            }
            if link_str.as_str() == end_page {
                println!("Done");
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }
    let mut cur = end_page;
    let mut path: VecDeque<String> = VecDeque::new();
    loop {
        let parent = seen.get(cur).unwrap();
        if parent.0 != cur {
            if let Some(redirect_str) = &parent.1{
                path.push_front(String::from(format!("{} (Redirected from: {})", cur, redirect_str)));
            }else{
                path.push_front(String::from(cur));
            }
            cur = parent.0.as_str();
        } else {
            path.push_front(String::from(cur));
            break;
        }
    }
    let search_end = search_start.elapsed();
    println!("{:?}", path);
    println!("Path found in {:?}", search_end);
    Ok(())
}

fn main() {
    println!("Wikimapia v0.1.0. Enter 'h' for list of commands");
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "h" => {
                println!("h          Displays this message");
                println!("reseed     Re seeds database of connections");
                println!("search     Starts shortest path search between articles");
                println!("exit       Exits application")
            }
            "reseed" => {
                if let Err(e) = seed_db() {
                    eprintln!("Error seeding database: {}", e);
                }
            }
            "search" => {
                print!("Enter start page: ");
                std::io::stdout().flush().unwrap();
                let mut start_page = String::new();
                std::io::stdin().read_line(&mut start_page).unwrap();
                let start_page = start_page.trim();
                if let Err(e) = check_for_page(start_page) {
                    eprintln!("Page {} doesn't exist: {}", start_page, e);
                    continue;
                }

                print!("Enter end page: ");
                std::io::stdout().flush().unwrap();
                let mut end_page = String::new();
                std::io::stdin().read_line(&mut end_page).unwrap();
                let end_page = end_page.trim();
                if let Err(e) = check_for_page(end_page) {
                    eprintln!("Page {} doesn't exist: {}", end_page, e);
                    continue;
                }

                let path = find_shortest_path(start_page, end_page);
            }
            "depth" => {
                print!("Enter start page: ");
                std::io::stdout().flush().unwrap();
                let mut start_page = String::new();
                std::io::stdin().read_line(&mut start_page).unwrap();
                let start_page = start_page.trim();
                if let Err(e) = check_for_page(start_page) {
                    eprintln!("Page {} doesn't exist: {}", start_page, e);
                    continue;
                }

                let _ = find_depth(start_page);
            }
            "exit" => break,
            _ => println!("Invalid input, enter 'h' for list of commands."),
        }
    }
}
