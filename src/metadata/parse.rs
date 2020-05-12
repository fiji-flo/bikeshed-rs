use chrono::format::ParseResult;

use super::metadata::Date;

pub fn parse_date(val: &String) -> ParseResult<Date> {
    if val == "now" {
        Ok(chrono::offset::Utc::now().naive_utc().date())
    } else {
        match Date::parse_from_str(val, "%Y-%m-%d") {
            Ok(date) => Ok(date),
            Err(err) => Err(err),
        }
    }
}

pub fn parse_editor(val: &String) -> Vec<String> {
    // TODO: parse editor
    let mut vec = Vec::new();
    vec.push(String::from("TODO: parse editor") + " " + val);
    vec
}

pub fn parse_level(val: &String) -> String {
    if val == "none" {
        String::new()
    } else {
        val.clone()
    }
}

pub fn parse_vec(val: &String) -> Vec<String> {
    let mut vec = Vec::new();
    vec.push(val.clone());
    vec
}
