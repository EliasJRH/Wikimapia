//
use regex::RegexBuilder;

// Simple function that takes a string and returns the same string with the first letter capitalized
pub fn capitalize_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// Checks strings against strings to make sure they don't link to namespace pages
pub fn process_article_name<'a>(name: &'a str) -> Option<&'a str> {
    if name.starts_with(":") {
        return None;
    }

    let namespace_regex = RegexBuilder::new(r"\w*:\S\w*")
        .case_insensitive(true)
        .build()
        .unwrap();

    let mut split = name.split("|");
    if let Some(processed_name) = split.next() {
        if namespace_regex.is_match(&(processed_name.split(" ").next().unwrap())) {
            return None;
        }
        return Some(&processed_name);
    }
    return None;
}
