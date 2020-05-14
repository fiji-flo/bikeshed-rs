use regex::Regex;

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
    pub w3c_id: Option<String>,
}

impl Editor {
    pub fn new(name: String) -> Self {
        Editor {
            name,
            ..Default::default()
        }
    }
}

pub fn parse_editor(val: &str) -> Vec<Editor> {
    // Editor: <editor-name> [<w3c-id> | <affiliation> | <email> | <homepage>]*

    lazy_static! {
        // w3c id
        static ref W3C_ID_REG: Regex = Regex::new(r"w3cid \d+$").unwrap();
    }

    let mut pieces = val
        .split(",")
        .map(|piece| piece.trim())
        .collect::<Vec<&str>>();
    let mut editor = Editor::new(String::from(pieces[0]));

    pieces = pieces[1..]
        .iter()
        .cloned()
        .filter(|piece| {
            if W3C_ID_REG.is_match(piece) && editor.w3c_id.is_none() {
                editor.w3c_id = Some(String::from(&piece[6..]));
                false
            } else {
                true
            }
        })
        .collect::<Vec<&str>>();

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
