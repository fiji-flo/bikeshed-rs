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

#[derive(Debug, Clone, Default)]
pub struct Editor {
    pub name: String,
}

impl Editor {
    pub fn new(name: String) -> Self {
        Editor { name }
    }
}

pub fn parse_editor(val: &str) -> Vec<Editor> {
    // Editor: <editor-name> [<w3c-id> | <affiliation> | <email> | <homepage>]*
    let pieces = val.split(",").collect::<Vec<&str>>();
    let editor = Editor::new(String::from(pieces[0]));
    vec![editor]
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
