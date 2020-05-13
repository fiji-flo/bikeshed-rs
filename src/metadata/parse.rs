use crate::util::date::{Date, ParseResult};

pub fn parse_date(val: &str) -> ParseResult {
    if val == "now" {
        Ok(Date::now())
    } else {
        match Date::parse_from_str(val, "%Y-%m-%d") {
            Ok(date) => Ok(date),
            Err(err) => Err(err),
        }
    }
}

pub fn parse_editor(val: &str) -> Vec<String> {
    // TODO: parse editor
    vec![String::from("TODO: parse editor") + " " + val]
}

pub fn parse_level(val: &str) -> String {
    if val == "none" {
        String::new()
    } else {
        val.to_owned()
    }
}

pub fn parse_vec(val: &str) -> Vec<String> {
    vec![val.to_owned()]
}
