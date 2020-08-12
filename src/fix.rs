use regex::{Captures, Regex};
use std::collections::{HashMap, VecDeque};

use crate::config::{DFN_SELECTOR, DFN_TYPES, LINK_TYPES};
use crate::html;
use crate::spec::Spec;
use crate::util;

// Code span manager would protect code spans from map functions.

#[derive(Debug, Default)]
pub struct CodeSpanManager {
    text_pieces: VecDeque<String>,
    code_pieces: VecDeque<String>,
}

impl CodeSpanManager {
    pub fn new(text: String) -> Self {
        lazy_static! {
            static ref REG: Regex = Regex::new(
                r"(?x)
                (?P<escape>\\`)
                |(?P<inner_text>[\w-]*)(?P<backticks>`+)"
            )
            .unwrap();
        }

        enum Mode {
            Text,
            Code,
        }

        let mut text_pieces = VecDeque::new();
        let mut code_pieces = VecDeque::new();

        let mut curr_mode = Mode::Text;
        let mut curr_index = 0;

        let mut backtick_count = 0;

        for mat in REG.find_iter(&text) {
            let start = mat.start();
            let end = mat.end();

            let caps = REG.captures(&text[start..end]).unwrap();

            match curr_mode {
                Mode::Text => {
                    if caps.name("escape").is_some() {
                        let text_piece = text[curr_index..start].to_owned() + "`";
                        text_pieces.push_back(text_piece);

                        code_pieces.push_back("".to_owned());
                        curr_index = end;
                    } else {
                        let text_piece = text[curr_index..start].to_owned();
                        text_pieces.push_back(text_piece);

                        backtick_count = caps["backticks"].len();
                        curr_index = end;
                        curr_mode = Mode::Code;
                    }
                }
                Mode::Code => {
                    if caps.name("escape").is_some() {
                        continue;
                    }

                    if caps["backticks"].len() == backtick_count {
                        let inner_text = text[curr_index..start].to_owned() + &caps["inner_text"];
                        code_pieces.push_back(inner_text);

                        curr_index = end;
                        curr_mode = Mode::Text;
                    }
                }
            }
        }

        // Handle the last piece.
        match curr_mode {
            Mode::Text => {
                let text_piece = text[curr_index..].to_owned();
                text_pieces.push_back(text_piece);
            }
            Mode::Code => {
                let text_piece = "`".repeat(backtick_count) + &text[curr_index..];
                text_pieces.push_back(text_piece);
            }
        }

        CodeSpanManager {
            text_pieces,
            code_pieces,
        }
    }

    pub fn map_text_pieces(&mut self, map_fn: impl Fn(&str) -> String) {
        self.text_pieces = self
            .text_pieces
            .iter()
            .map(|piece| map_fn(&piece))
            .collect();
    }

    pub fn extract(&mut self) -> String {
        // Zip text pieces and code pieces.
        let mut zipped = String::new();

        while let Some(text_piece) = self.text_pieces.pop_front() {
            zipped += &text_piece;

            if let Some(code_piece) = self.code_pieces.pop_front() {
                if !code_piece.is_empty() {
                    zipped += &format!("<code>{}</code>", html::escape_html(code_piece));
                }
            }
        }

        zipped
    }
}

// Replace macros with text.
pub fn replace_macros<'a>(text: &str, macros: &HashMap<&'a str, String>) -> String {
    lazy_static! {
        static ref REG: Regex = Regex::new(
            r"(?x)
            \[
            (?P<inner_text>[A-Z0-9-]+)
            \]"
        )
        .unwrap();
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
        let left = &caps["left"];
        let right = &caps["right"];
        format!("{}’{}", left, right)
    };

    util::regex::replace_all(&REG, text, replacer)
}

pub fn canonicalize_shortcuts(doc: &Spec) {
    // Process dfn type.
    for dfn_el in html::select(doc.dom(), &DFN_SELECTOR) {
        for dfn_type in DFN_TYPES.iter() {
            if let Some(attr_val) = html::get_attr_val(&dfn_el, dfn_type) {
                if attr_val.is_empty() {
                    html::remove_attr(&dfn_el, dfn_type);
                    html::insert_attr(&dfn_el, "data-dfn-type", dfn_type.to_owned());
                    break;
                }
            }
        }
    }

    // Process link type.
    for a_el in html::select(doc.dom(), "a") {
        for link_type in LINK_TYPES.iter() {
            if let Some(attr_val) = html::get_attr_val(&a_el, link_type) {
                if attr_val.is_empty() {
                    html::remove_attr(&a_el, link_type);
                    html::insert_attr(&a_el, "data-link-type", link_type.to_owned());
                    break;
                }
            }
        }
    }

    // Process "for" values.
    for el in html::select(doc.dom(), &format!("{}, a", DFN_SELECTOR.as_str())) {
        let for_val = match html::get_attr_val(&el, "for") {
            Some(for_val) => for_val,
            None => continue,
        };

        match html::get_tag(&el).unwrap().as_str() {
            "a" => html::insert_attr(&el, "data-link-for", for_val),
            _ => html::insert_attr(&el, "data-dfn-for", for_val),
        };

        html::remove_attr(&el, "for");
    }
}
