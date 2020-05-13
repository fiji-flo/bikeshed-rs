use chrono::{Datelike, NaiveDate};
use regex::Regex;
use titlecase::titlecase;

use super::join::Joinable;
use super::parse;
use crate::config::SHORT_TO_LONG_STATUS;
use crate::line::Line;
use crate::spec::Spec;

pub type Date = NaiveDate;

#[derive(Debug, Default, Clone)]
pub struct Metadata {
    pub has_keys: bool,
    pub abs: Vec<String>,
    pub canonical_url: Option<String>,
    pub date: Option<Date>,
    pub ed: Option<String>,
    pub editors: Vec<String>,
    pub group: Option<String>,
    pub level: Option<String>,
    pub shortname: Option<String>,
    pub raw_status: Option<String>,
    pub title: Option<String>,
}

impl Metadata {
    pub fn new() -> Metadata {
        Self::default()
    }

    pub fn join_all(sources: &[&Metadata]) -> Metadata {
        let mut md = Metadata::new();
        for source in sources {
            md.join(source);
        }
        md
    }

    pub fn add_data(&mut self, key: &str, val: &str, line_num: Option<u32>) {
        let mut key = key.trim().to_string();

        if key != "ED" && key != "TR" && key != "URL" {
            key = titlecase(&key);
        }

        match key.as_str() {
            "Abstract" => {
                let val = parse::parse_vec(val);
                self.abs.extend(val);
            }
            "Canonical Url" => {
                let val = val.to_owned();
                self.canonical_url = Some(val);
            }
            "Date" => {
                let val = match parse::parse_date(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("The \"Date\" field must be in the format YYYY-MM-DD."; line_num)
                    }
                };
                self.date = Some(val);
            }
            "ED" => {
                let val = val.to_owned();
                self.ed = Some(val);
            }
            "Editor" => {
                let val = parse::parse_editor(val);
                self.editors.extend(val);
            }
            "Group" => {
                let val = val.to_owned();
                self.group = Some(val);
            }
            "Level" => {
                let val = parse::parse_level(val);
                self.level = Some(val);
            }
            "Shortname" => {
                let val = val.to_owned();
                self.shortname = Some(val);
            }
            "Status" => {
                let val = val.to_owned();
                self.raw_status = Some(val);
            }
            "Title" => {
                let val = val.to_owned();
                self.title = Some(val);
            }
            _ => die!("Unknown metadata key \"{}\".", key; line_num),
        }

        self.has_keys = true;
    }

    pub fn fill_macros(&self, doc: &mut Spec) {
        let macros = &mut doc.macros;

        if let Some(ref date) = self.date {
            macros.insert(
                "date",
                date.format(&format!("{} %B %Y", date.day())).to_string(),
            );
            macros.insert("isodate", date.to_string());
        }
        if let Some(ref level) = self.level {
            macros.insert("level", level.clone());
        }
        if let Some(ref shortname) = self.shortname {
            macros.insert("shortname", shortname.clone());
        }
        if let Some(ref raw_status) = self.raw_status {
            macros.insert(
                "longstatus",
                SHORT_TO_LONG_STATUS
                    .get(raw_status.as_str())
                    .unwrap()
                    .to_string(),
            );
        }
        if let Some(ref title) = self.title {
            macros.insert("title", title.clone());
            macros.insert("spectitle", title.clone());
        }
    }

    pub fn compute_implicit_metadata(&mut self) {
        if self.canonical_url.as_ref().map_or(true, |url| url == "ED") {
            self.canonical_url = self.ed.clone();
        }
    }

    pub fn validate(&self) {
        if !self.has_keys {
            die!("No metadata provided.");
        }
    }
}

// TODO(#3): Figure out if we can get rid of this html-parsing-with-regexes.
pub fn parse_metadata(lines: &[Line]) -> (Metadata, Vec<Line>) {
    lazy_static! {
        // title reg
        static ref TITLE_REG: Regex = Regex::new(r"\s*<h1[^>]*>(.*?)</h1>").unwrap();
        // begin tag reg
        static ref BEGIN_TAG_REG: Regex = Regex::new(r"<(pre|xmp) [^>]*class=[^>]*metadata[^>]*>").unwrap();
        // </pre> end tag
        static ref PRE_END_TAG: Regex = Regex::new(r"</pre>\s*").unwrap();
        // </xmp> end tag
        static ref XMP_END_TAG: Regex = Regex::new(r"</xmp>\s*").unwrap();
        // pair reg
        static ref PAIR_REG: Regex = Regex::new(r"([^:]+):\s*(.*)").unwrap();
    }

    let mut md = Metadata::new();
    let mut new_lines: Vec<Line> = Vec::new();
    let mut in_metadata = false;
    let mut last_key: Option<&str> = None;
    let mut end_tag_reg: Option<&Regex> = None;

    for line in lines {
        if !in_metadata && BEGIN_TAG_REG.is_match(&line.text) {
            // handle begin tag
            in_metadata = true;
            md.has_keys = true;
            if line.text.starts_with("<pre") {
                end_tag_reg = Some(&PRE_END_TAG);
            } else {
                end_tag_reg = Some(&XMP_END_TAG);
            }
        } else if in_metadata && end_tag_reg.unwrap().is_match(&line.text) {
            // handle end tag
            in_metadata = false;
        } else if in_metadata {
            if last_key.is_some() && line.text.trim().is_empty() {
                // if the line is empty, continue the previous key
                md.add_data(last_key.unwrap(), &line.text, Some(line.index));
            } else if PAIR_REG.is_match(&line.text) {
                // handle key-val pair
                let caps = PAIR_REG.captures(&line.text).unwrap();
                let key = caps.get(1).map_or("", |k| k.as_str());
                let val = caps.get(2).map_or("", |v| v.as_str());
                md.add_data(key, val, Some(line.index));
                last_key = Some(key);
            } else {
                // wrong key-val pair
                die!("Incorrectly formatted metadata"; Some(line.index));
            }
        } else if TITLE_REG.is_match(&line.text) {
            // handle title
            if md.title.is_none() {
                let caps = TITLE_REG.captures(&line.text).unwrap();
                let title = caps.get(1).map_or("", |m| m.as_str());
                md.add_data("Title", title, Some(line.index));
            }
            new_lines.push(line.clone());
        } else {
            // handle lines that do not contain metadata
            new_lines.push(line.clone());
        }
    }

    (md, new_lines)
}
