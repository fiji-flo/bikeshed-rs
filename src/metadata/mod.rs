pub mod parse;

use indexmap::map::IndexMap;
use regex::Regex;
use serde_json::map::Map;
use serde_json::{self, Value};
use titlecase::titlecase;

use self::parse::{Editor, EditorTerm};
use crate::config::SHORT_TO_LONG_STATUS;
use crate::line::Line;
use crate::markdown;
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
    pub indent: Option<u32>,
    pub infer_css_dfns: Option<bool>,
    pub markup_shorthands: BoolSet<String>,
    pub remove_multiple_links: Option<bool>,
    pub title: Option<String>,
    pub tr: Option<String>,
    pub work_status: Option<String>,
    // custom metadata
    pub custom_md: IndexMap<String, Vec<String>>,
}

impl Metadata {
    pub fn new() -> Self {
        Metadata {
            boilerplate: BoolSet::new_with_default(true),
            markup_shorthands: BoolSet::new_with_default(false),
            ..Default::default()
        }
    }

    pub fn from_json(data: String) -> Self {
        let mut md = Metadata::new();
        let obj: Map<String, Value> = match serde_json::from_str(&data) {
            Ok(Value::Object(obj)) => obj,
            _ => die!("Fail to load JSON:\n{}", data),
        };

        for (key, val) in obj.iter() {
            match val {
                Value::String(str_val) => md.add_data(key, str_val, None),
                Value::Array(arr_val) => {
                    for indiv_val in arr_val {
                        match indiv_val {
                            Value::String(str_val) => md.add_data(key, str_val, None),
                            _ => die!(
                                concat!(
                                    "JSON metadata values must be strings or arrays of strings. ",
                                    "\"{0}\" is something else."
                                ),
                                key
                            ),
                        }
                    }
                }
                _ => die!(
                    concat!(
                        "JSON metadata values must be strings or arrays of strings. ",
                        "\"{0}\" is something else."
                    ),
                    key
                ),
            }
        }

        md
    }

    pub fn add_data(&mut self, key: &str, val: &str, line_num: Option<u32>) {
        let mut key = key.trim().to_string();

        if key != "ED" && key != "TR" && key != "URL" {
            key = titlecase(&key);
        }

        if key.starts_with("!") {
            let key = &key[1..];
            self.custom_md
                .entry(key.to_owned())
                .or_insert(Vec::new())
                .push(val.to_owned());
            return;
        }

        match key.as_str() {
            "Abstract" => {
                let val = val.to_owned();
                self.abs.push(val);
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
                    Err(_) => die!(
                        concat!(
                            "Boilerplate metadata pieces are a boilerplate label and a boolean. ",
                            "Got: {}."
                        ),
                        val; line_num
                    ),
                };
                self.boilerplate.update(&val);
            }
            "Canonical URL" => {
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
                    Err(_) => die!(
                        concat!(
                            "\"Editor\" format is \"<name>, <affiliation>?, <email-or-contact-page>?\". ",
                            "Got: {}."
                        ),
                        val; line_num
                    ),
                };
                self.editors.push(val);
            }
            "Editor Term" => {
                let val = match parse::parse_editor_term(val) {
                    Ok(val) => val,
                    Err(_) => die!(
                        concat!(
                            "\"Editor Term\" format is \"<singular-term>, <plural-term>\". ",
                            "Got: {}."
                        ),
                        val; line_num
                    ),
                };
                self.editor_term = Some(val);
            }
            "Group" => {
                let val = val.to_owned();
                self.group = Some(val);
            }
            "Indent" => {
                let val = match parse::parse_natural_number(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("\"Indent\" field must be natural number. Got: {}", val; line_num)
                    }
                };
                self.indent = Some(val);
            }
            "Markup Shorthands" => {
                let val = match parse::parse_markup_shorthands(val) {
                    Ok(val) => val,
                    Err(_) => die!(
                        concat!(
                            "Markup shorthands metadata pieces are a markup markup shorthand category ",
                            "and a boolean. Got: {}."
                        ),
                        val; line_num
                    ),
                };
                self.markup_shorthands.update(&val);
            }
            "Infer CSS Dfns" => {
                let val = match parse::parse_bool(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("The \"Infer CSS Dfns\" field must be boolish. Got: {}.", val; line_num)
                    }
                };
                self.infer_css_dfns = Some(val);
            }
            "Remove Multiple Links" => {
                let val = match parse::parse_bool(val) {
                    Ok(val) => val,
                    Err(_) => {
                        die!("The \"Remove Multiple Links\" field must be boolish. Got: {}.", val; line_num)
                    }
                };
                self.remove_multiple_links = Some(val);
            }
            "Title" => {
                let val = val.to_owned();
                self.title = Some(val);
            }
            "TR" => {
                let val = val.to_owned();
                self.tr = Some(val);
            }
            "Work Status" => {
                let val = match parse::parse_work_status(val) {
                    Ok(val) => val,
                    Err(_) => die!(
                        concat!(
                            "Work Status must be one of (completed, stable, testing, refining, ",
                            "revising, exploring, rewriting, abandoned). Got: {}."
                        ),
                        val; line_num
                    ),
                };
                self.work_status = Some(val);
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
        // Indent
        if other.indent.is_some() {
            self.indent = other.indent;
        }
        // Infer CSS Dfns
        if other.infer_css_dfns.is_some() {
            self.infer_css_dfns = other.infer_css_dfns;
        }
        // Markup Shorthands
        self.markup_shorthands.update(&other.markup_shorthands);
        // Remove Multiple Links
        if other.remove_multiple_links.is_some() {
            self.remove_multiple_links = other.remove_multiple_links;
        }
        // Title
        if other.title.is_some() {
            self.title = other.title;
        }
        // TR
        if other.tr.is_some() {
            self.tr = other.tr;
        }
        // Work Status
        if other.work_status.is_some() {
            self.work_status = other.work_status;
        }
        // Custom Metadata
        self.custom_md.extend(other.custom_md);
    }

    pub fn fill_macros(&self, doc: &mut Spec) {
        let macros = &mut doc.macros;

        // abstract
        macros.insert(
            "abstract",
            markdown::parse(&self.abs, self.indent()).join("\n"),
        );
        // level
        if let Some(ref level) = self.level {
            macros.insert("level", level.clone());
        }
        // shortname & vshortname
        if let Some(ref shortname) = self.shortname {
            macros.insert("shortname", shortname.clone());

            let vshortname = if let Some(ref level) = self.level {
                format!("{}-{}", shortname, level)
            } else {
                shortname.clone()
            };
            macros.insert("vshortname", vshortname);
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
        // version
        if let Some(ref ed) = self.ed {
            macros.insert("version", ed.clone());
        }
        // title & spectitle
        if let Some(ref title) = self.title {
            macros.insert("title", title.clone());
            macros.insert("spectitle", title.clone());
        }
        // work status
        if let Some(ref work_status) = self.work_status {
            macros.insert("workstatus", work_status.clone());
        }
    }

    pub fn compute_implicit_metadata(&mut self) {
        if self.canonical_url.as_ref().map_or(true, |url| url == "TR") && self.tr.is_some() {
            self.canonical_url = self.tr.clone();
        } else if self.canonical_url.as_ref().map_or(true, |url| url == "ED") && self.ed.is_some() {
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

    pub fn indent(&self) -> u32 {
        self.indent.unwrap_or(4)
    }
}

// Join all "group" field of metadata.
pub fn extract_group(mds: &[&Metadata]) -> Option<String> {
    for md in mds.iter().rev() {
        if md.group.is_some() {
            return md.group.clone();
        }
    }
    None
}

// Join all "raw status" field of metadata.
pub fn extract_status(mds: &[&Metadata]) -> Option<String> {
    for md in mds.iter().rev() {
        if md.raw_status.is_some() {
            return md.raw_status.clone();
        }
    }
    None
}

// TODO(#3): Figure out if we can get rid of this html-parsing-with-regexes.
pub fn parse_metadata(lines: &[Line]) -> (Metadata, Vec<Line>) {
    lazy_static! {
        // regex for title
        static ref TITLE_REG: Regex = Regex::new(r"\s*<h1[^>]*>(?P<title>.*?)</h1>").unwrap();
        // regex for begin tag
        static ref BEGIN_TAG_REG: Regex = Regex::new(r"<(pre|xmp) [^>]*class=[^>]*metadata[^>]*>").unwrap();
        // regex for </pre> end tag
        static ref PRE_END_TAG_REG: Regex = Regex::new(r"</pre>\s*").unwrap();
        // regex for </xmp> end tag
        static ref XMP_END_TAG_REG: Regex = Regex::new(r"</xmp>\s*").unwrap();
        // regex for line that starts with spaces
        static ref START_WITH_SPACES_REG: Regex = Regex::new(r"^\s+").unwrap();
        // regex for key-value pair
        static ref PAIR_REG: Regex = Regex::new(r"(?P<key>[^:]+):\s*(?P<val>.*)").unwrap();
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
                end_tag_reg = Some(&PRE_END_TAG_REG);
            } else {
                end_tag_reg = Some(&XMP_END_TAG_REG);
            }
        } else if in_metadata && end_tag_reg.unwrap().is_match(&line.text) {
            // handle end tag
            in_metadata = false;
        } else if in_metadata {
            if last_key.is_some()
                && (line.text.trim().is_empty() || START_WITH_SPACES_REG.is_match(&line.text))
            {
                // if the line is empty or starts with 1+ spaces, continue the previous key
                md.add_data(last_key.unwrap(), &line.text, Some(line.index));
            } else if let Some(caps) = PAIR_REG.captures(&line.text) {
                // handle key-val pair
                let key = caps.name("key").unwrap().as_str();
                let val = caps.name("val").unwrap().as_str();
                md.add_data(key, val, Some(line.index));
                last_key = Some(key);
            } else {
                // wrong key-val pair
                die!("Incorrectly formatted metadata."; Some(line.index));
            }
        } else if let Some(caps) = TITLE_REG.captures(&line.text) {
            // handle title
            if md.title.is_none() {
                let title = caps.name("title").unwrap().as_str();
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
