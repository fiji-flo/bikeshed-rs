use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::collections::{BTreeMap, HashMap};
use std::fs;

use crate::boilerplate;
use crate::config::SOURCE_FILE_EXTENSIONS;
use crate::html;
use crate::line::Line;
use crate::metadata::metadata::{self, MetadataManager};
use crate::util::reader;

#[derive(Debug, Default)]
pub struct Spec<'a> {
    infile: &'a str,
    lines: Vec<Line>,
    pub mm: MetadataManager,
    mm_baseline: MetadataManager,
    mm_document: MetadataManager,
    mm_command_line: MetadataManager,
    pub macros: HashMap<&'static str, String>,
    html: String,
    pub document: Option<NodeRef>,
    pub head: Option<NodeRef>,
    pub body: Option<NodeRef>,
    pub extra_styles: BTreeMap<&'static str, &'static str>,
}

impl<'a> Spec<'a> {
    pub fn new(infile: &str) -> Spec {
        let lines = Spec::read_lines_from_source(infile);

        let mut mm_baseline = MetadataManager::new();
        mm_baseline.add_data("Date", &String::from("now"), None);

        let extra_styles = btreemap! {
            "md-lists" => include_str!("style/md-lists.css"),
            "autolinks" =>  include_str!("style/autolinks.css"),
            "selflinks" => include_str!("style/selflinks.css"),
            "counters" => include_str!("style/counters.css"),
        };

        Spec {
            infile: infile,
            lines: lines,
            mm: MetadataManager::new(),
            mm_baseline: mm_baseline,
            mm_document: MetadataManager::new(),
            mm_command_line: MetadataManager::new(),
            extra_styles: extra_styles,
            ..Default::default()
        }
    }

    fn read_lines_from_source(infile: &str) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        if let Ok(src_lines) = reader::read_lines(infile) {
            for (index, src_line) in src_lines.enumerate() {
                if let Ok(text) = src_line {
                    lines.push(Line {
                        index: 1 + (index as u32),
                        text: text,
                    });
                }
            }
        }
        lines
    }

    pub fn preprocess(&mut self) {
        self.assemble_document();
        self.process_document();
    }

    fn assemble_document(&mut self) {
        let (mm_document, lines) = metadata::parse_metadata(&self.lines);
        self.mm_document = mm_document;
        self.lines = lines;

        let mut mm = MetadataManager::join_all(&[
            &self.mm_baseline,
            &self.mm_document,
            &self.mm_command_line,
        ]);
        mm.compute_implicit_metadata();
        mm.fill_macros(self);
        mm.validate();
        self.mm = mm;

        self.html = self
            .lines
            .iter()
            .map(|l| l.text.clone())
            .collect::<Vec<String>>()
            .join("\n");
        boilerplate::add_header_footer(&mut self.html);
        self.html = html::helper::replace_macros(&self.html, &self.macros);

        self.document = Some(kuchiki::parse_html().one(self.html.clone()));
        if let Ok(head) = self.document.as_ref().unwrap().select_first("head") {
            self.head = Some(head.as_node().clone());
        }
        if let Ok(body) = self.document.as_ref().unwrap().select_first("body") {
            self.body = Some(body.as_node().clone());
        }
    }

    fn process_document(&mut self) {
        boilerplate::add_canonical_url(self);
        boilerplate::add_bikeshed_boilerplate(self);
    }

    pub fn finish(&self, outfile: Option<&str>) {
        if let Some(document) = &self.document {
            let outfile = self.handle_outfile(outfile);
            let rendered = document.to_string();
            fs::write(outfile, rendered).expect("unable to write file");
        }
    }

    fn handle_outfile(&self, outfile: Option<&str>) -> String {
        if outfile.is_some() {
            outfile.unwrap().to_string()
        } else {
            for extension in SOURCE_FILE_EXTENSIONS.iter() {
                if self.infile.ends_with(extension) {
                    return (&self.infile[..self.infile.len() - extension.len()]).to_string()
                        + ".html";
                }
            }
            String::from("-")
        }
    }
}
