use regex::Regex;
use titlecase::titlecase;

use super::parse::{self, Editor, EditorTerm};
use crate::config::SHORT_TO_LONG_STATUS;
use crate::line::Line;
use crate::spec::Spec;
use crate::util::boolset::BoolSet;
use crate::util::date::Date;

#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub has_keys: bool,
    // required metadata
    pub abs: Vec<String>,
    pub ed: Option<String>,
    pub level: Option<String>,
    pub shortname: Option<String>,
    pub raw_status: Option<String>,
    // optional metadata
    pub boilerplate: BoolSet<String>,
    pub canonical_url: Option<String>,
    pub date: Date,
    pub editors: Vec<Editor>,
    pub editor_term: Option<EditorTerm>,
    pub group: Option<String>,
    pub title: Option<String>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            boilerplate: BoolSet::new_with_default(true),
            ..Default::default()
        }
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
            "ED" => {
                let val = val.to_owned();
                self.ed = Some(val);
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
            "Boilerplate" => {
                let val = match parse::parse_boilerplate(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("Boilerplate metadata pieces are a boilerplate label and a boolean. Got: {}.", val; line_num)
                    }
                };
                self.boilerplate.update(&val);
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
                self.date = val;
            }
            "Editor" => {
                let val = match parse::parse_editor(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("\"Editor\" format is \"<name>, <affiliation>?, <email-or-contact-page>?\". Got: {}.", val; line_num)
                    }
                };
                self.editors.push(val);
            }
            "Editor Term" => {
                let val = match parse::parse_editor_term(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("\"Editor Term\" format is \"<singular-term>, <plural-term>\". Got: {}.", val; line_num)
                    }
                };
                self.editor_term = Some(val);
            }
            "Group" => {
                let val = val.to_owned();
                self.group = Some(val);
            }
            "Title" => {
                let val = val.to_owned();
                self.title = Some(val);
            }
            _ => die!("Unknown metadata key \"{}\".", key; line_num),
        }

        self.has_keys = true;
    }

    pub fn join(&mut self, other: Self) {
        if other.has_keys {
            self.has_keys = true;
        } else {
            return;
        }

        // Abstract
        self.abs.extend(other.abs.into_iter());
        // ED
        if other.ed.is_some() {
            self.ed = other.ed;
        }
        // Level
        if other.level.is_some() {
            self.level = other.level;
        }
        // Shortname
        if other.shortname.is_some() {
            self.shortname = other.shortname;
        }
        // Status
        if other.raw_status.is_some() {
            self.raw_status = other.raw_status;
        }
        // Boilerplate
        self.boilerplate.update(&other.boilerplate);
        // Canonical Url
        if other.canonical_url.is_some() {
            self.canonical_url = other.canonical_url;
        }
        // Date
        self.date = other.date;
        // Editor
        self.editors.extend(other.editors.into_iter());
        // Editor Term
        if other.editor_term.is_some() {
            self.editor_term = other.editor_term;
        }
        // Group
        if other.group.is_some() {
            self.group = other.group;
        }
        // Title
        if other.title.is_some() {
            self.title = other.title;
        }
    }

    pub fn fill_macros(&self, doc: &mut Spec) {
        let macros = &mut doc.macros;

        // level
        if let Some(ref level) = self.level {
            macros.insert("level", level.clone());
        }
        // shortname
        if let Some(ref shortname) = self.shortname {
            macros.insert("shortname", shortname.clone());
        }
        // longstatus
        if let Some(ref raw_status) = self.raw_status {
            macros.insert(
                "longstatus",
                SHORT_TO_LONG_STATUS
                    .get(raw_status.as_str())
                    .unwrap()
                    .to_string(),
            );
        }
        // date
        macros.insert(
            "date",
            self.date
                .format(&format!("{} %B %Y", self.date.day()))
                .to_string(),
        );
        // isodate
        macros.insert("isodate", self.date.to_string());
        // title & spectitle
        if let Some(ref title) = self.title {
            macros.insert("title", title.clone());
            macros.insert("spectitle", title.clone());
        }
    }

    pub fn compute_implicit_metadata(&mut self) {
        if self.canonical_url.as_ref().map_or(true, |url| url == "ED") {
            self.canonical_url = self.ed.clone();
        }
        if self.editor_term.is_none() {
            self.editor_term = Some(EditorTerm::default());
        }
    }

    pub fn validate(&self) {
        if !self.has_keys {
            die!("No metadata provided.");
        }
    }
}

// TODO(#3): figure out if we can get rid of this html-parsing-with-regexes
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
                die!("Incorrectly formatted metadata."; Some(line.index));
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
