use bzip2::read::BzDecoder;
use rusqlite::Result;
use scraper::{Html, Selector};
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::time::{Duration, Instant};
use std::usize;

// Takes contents of a file (wikipedia dumps) and breaks it up into <divisons> sections
// divisions is the number of threads available
pub fn divide_input(contents_file: File, divisions: Option<usize>) -> Vec<String> {
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

// Gets name of all current wikipedia dumps
pub fn get_wikipedia_dumps() -> Result<VecDeque<String>, Box<dyn std::error::Error>> {
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

// Downloads file from wikipedia dump website with file name <file_name>
pub fn download_decompress_save_to_file(file_name: &String) -> Result<File, std::io::Error> {
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
