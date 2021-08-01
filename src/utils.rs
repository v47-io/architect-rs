use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref ID_REGEX: Regex = Regex::new("[a-zA-Z_][a-zA-Z0-9_$]*").unwrap();
}

pub fn is_identifier(value: &str) -> bool {
    ID_REGEX.is_match(value)
}
