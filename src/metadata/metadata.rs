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
pub struct MetadataManager {
    pub has_metadata: bool,
    pub abs: Option<Vec<String>>,
    pub canonical_url: Option<String>,
    pub date: Option<Date>,
    pub ed: Option<String>,
    pub editors: Option<Vec<String>>,
    pub group: Option<String>,
    pub level: Option<String>,
    pub shortname: Option<String>,
    pub raw_status: Option<String>,
    pub title: Option<String>,
}

impl MetadataManager {
    pub fn new() -> MetadataManager {
        MetadataManager {
            has_metadata: false,
            ..Default::default()
        }
    }

    pub fn join_all(sources: &[&MetadataManager]) -> MetadataManager {
        let mut mm = MetadataManager::new();
        for source in sources {
            mm.join((*source).clone());
        }
        mm
    }

    pub fn add_data(&mut self, key: &str, val: &String, line_num: Option<u32>) {
        let mut key = key.trim().to_string();

        if key != "ED" && key != "TR" && key != "URL" {
            key = titlecase(&key);
        }

        match key.as_str() {
            "Abstract" => {
                let val = parse::parse_vec(val);
                self.abs.join(Some(val));
            }
            "Canonical Url" => {
                let val = val.clone();
                self.canonical_url.join(Some(val));
            }
            "Date" => {
                let val = match parse::parse_date(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("The \"Date\" field must be in the format YYYY-MM-DD."; line_num)
                    }
                };
                self.date.join(Some(val));
            }
            "ED" => {
                let val = val.clone();
                self.ed.join(Some(val));
            }
            "Editor" => {
                let val = parse::parse_editor(val);
                self.editors.join(Some(val));
            }
            "Group" => {
                let val = val.clone();
                self.group.join(Some(val));
            }
            "Level" => {
                let val = parse::parse_level(val);
                self.level.join(Some(val));
            }
            "Shortname" => {
                let val = val.clone();
                self.shortname.join(Some(val))
            }
            "Status" => {
                let val = val.clone();
                self.raw_status.join(Some(val));
            }
            "Title" => {
                let val = val.clone();
                self.title.join(Some(val));
            }
            _ => die!("Unknown metadata key \"{}\".", key; line_num),
        }

        self.has_metadata = true;
    }

    pub fn fill_macros(&self, doc: &mut Spec) {
        let macros = &mut doc.macros;

        if let Some(date) = self.date.as_ref() {
            macros.insert(
                "date",
                date.format(&format!("{} %B %Y", date.day())).to_string(),
            );
            macros.insert("isodate", date.to_string());
        }
        if let Some(level) = self.level.as_ref() {
            macros.insert("level", level.clone());
        }
        if let Some(shortname) = self.shortname.as_ref() {
            macros.insert("shortname", shortname.clone());
        }
        if let Some(raw_status) = self.raw_status.as_ref() {
            macros.insert(
                "longstatus",
                SHORT_TO_LONG_STATUS
                    .get(raw_status.as_str())
                    .unwrap()
                    .to_string(),
            );
        }
        if let Some(title) = self.title.as_ref() {
            macros.insert("title", title.clone());
            macros.insert("spectitle", title.clone());
        }
    }

    pub fn compute_implicit_metadata(&mut self) {
        if self.canonical_url.is_none() || self.canonical_url.as_ref().unwrap() == "ED" {
            self.canonical_url = self.ed.clone();
        }
    }

    pub fn validate(&self) {
        if !self.has_metadata {
            die!("The document requires at least one <pre class=metadata> block.");
        }
    }
}

pub fn parse_metadata(lines: &Vec<Line>) -> (MetadataManager, Vec<Line>) {
    let mut mm = MetadataManager::new();
    let mut new_lines: Vec<Line> = Vec::new();
    let mut in_metadata = false;
    let mut last_key: Option<String> = None;

    let title_reg = Regex::new(r"\s*<h1[^>]*>(.*?)</h1>").unwrap();
    let begin_tag_reg = Regex::new(r"<(pre|xmp) [^>]*class=[^>]*metadata[^>]*>").unwrap();
    let mut end_tag_reg: Option<Regex> = None;
    let pair_reg = Regex::new(r"([^:]+):\s*(.*)").unwrap();

    for line in lines {
        if !in_metadata && begin_tag_reg.is_match(&line.text) {
            // handle begin tag
            in_metadata = true;
            mm.has_metadata = true;
            if line.text.starts_with("<pre") {
                end_tag_reg = Some(Regex::new(r"</pre>\s*").unwrap());
            } else {
                end_tag_reg = Some(Regex::new(r"</xmp>\s*").unwrap());
            }
        } else if in_metadata && end_tag_reg.as_mut().unwrap().is_match(&line.text) {
            // handle end tag
            in_metadata = false;
        } else if in_metadata {
            if last_key.is_some() && line.text.trim().is_empty() {
                // if the line is empty, continue the previous key
                mm.add_data(last_key.as_mut().unwrap(), &line.text, Some(line.index));
            } else if pair_reg.is_match(&line.text) {
                // handle key-val pair
                let caps = pair_reg.captures(&line.text).unwrap();
                let key = caps
                    .get(1)
                    .map_or(String::new(), |k| k.as_str().to_string());
                let val = caps
                    .get(2)
                    .map_or(String::new(), |v| v.as_str().to_string());
                mm.add_data(&key, &val, Some(line.index));
                last_key = Some(key);
            } else {
                // wrong key-val pair
                die!("Incorrectly formatted metadata"; Some(line.index));
            }
        } else if title_reg.is_match(&line.text) {
            // handle title
            if mm.title.is_none() {
                let caps = title_reg.captures(&line.text).unwrap();
                let title = caps
                    .get(1)
                    .map_or(String::new(), |m| m.as_str().to_string());
                mm.add_data(&String::from("Title"), &title, Some(line.index));
            }
            new_lines.push(line.clone());
        } else {
            // handle lines that do not contain metadata
            new_lines.push(line.clone());
        }
    }

    (mm, new_lines)
}