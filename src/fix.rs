use regex::{Captures, Regex};
use std::collections::HashMap;

use crate::util;

// Replace macros with text.
pub fn replace_macros<'a>(text: &str, macros: &HashMap<&'a str, String>) -> String {
    lazy_static! {
        static ref REG: Regex = Regex::new(r"\[(?P<inner_text>[A-Z0-9-]+)\]").unwrap();
    }

    let replacer = |caps: &Captures| -> String {
        let inner_text = caps["inner_text"].to_lowercase();

        if let Some(new_val) = macros.get(inner_text.as_str()) {
            new_val.to_owned()
        } else {
            caps["inner_text"].to_owned()
        }
    };

    util::regex::replace_all(&REG, text, replacer)
}

pub fn fix_typography(text: &str) -> String {
    lazy_static! {
        static ref REG: Regex = Regex::new(r"(?P<left>\w)'(?P<right>\w)").unwrap();
    }

    let replacer = |caps: &Captures| -> String {
        let left = caps.name("left").unwrap().as_str();
        let right = caps.name("right").unwrap().as_str();
        format!("{}’{}", left, right)
    };

    util::regex::replace_all(&REG, text, replacer)
}
