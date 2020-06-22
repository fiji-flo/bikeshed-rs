use regex::{Captures, Regex};
use std::collections::HashMap;

use crate::util;

// Replace macros with text.
pub fn replace_macros(text: &str, macros: &HashMap<&'static str, String>) -> String {
    lazy_static! {
        static ref REG: Regex = Regex::new(r"\[(?P<inner_text>[A-Z0-9-]+)\]").unwrap();
    }

    let replacer = |caps: &Captures| -> String {
        let inner_text = (&caps["inner_text"]).to_lowercase();
        if macros.contains_key(inner_text.as_str()) {
            macros.get(inner_text.as_str()).unwrap().to_string()
        } else {
            (&caps[0]).to_string()
        }
    };

    util::regex::replace_all(&REG, text, replacer)
}
