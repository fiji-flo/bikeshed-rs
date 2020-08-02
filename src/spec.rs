use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::collections::{BTreeMap, HashMap};
use std::fs;

use crate::boilerplate::{self, retrieve_boilerplate_with_info};
use crate::clean;
use crate::config::SOURCE_FILE_EXTENSIONS;
use crate::fix::{self, CodeSpanManager};
use crate::heading;
use crate::html;
use crate::line::Line;
use crate::link::biblio::manager::BiblioEntryManager;
use crate::link::reference::manager::ReferenceManager;
use crate::link::reference::Reference;
use crate::link::{self, biblio::BiblioEntry, dfn};
use crate::markdown;
use crate::metadata::{self, Metadata};
use crate::shorthand;
use crate::util::reader;

#[derive(Debug, Default)]
pub struct Spec<'a> {
    infile: &'a str,
    lines: Vec<Line>,
    pub md: Metadata,
    pub md_cli: Metadata,
    pub macros: HashMap<&'a str, String>,
    pub html: String,
    dom: Option<NodeRef>,
    head: Option<NodeRef>,
    body: Option<NodeRef>,
    pub containers: BTreeMap<String, NodeRef>,
    pub extra_styles: BTreeMap<&'a str, &'a str>,
    pub extra_scripts: BTreeMap<&'a str, &'a str>,
    pub reference_manager: ReferenceManager,
    pub biblio_entry_manager: BiblioEntryManager,
    // spec => (text => reference)
    pub external_references_used: HashMap<String, HashMap<String, Reference>>,
    // text => normative biblio entries
    pub normative_biblio_entries: HashMap<String, BiblioEntry>,
}

impl<'a> Spec<'a> {
    pub fn new(infile: &str, md_cli: Metadata) -> Spec {
        let lines = Spec::read_lines_from_source(infile);

        let extra_styles = btreemap! {
            "md-lists" => include_str!("style/md-lists.css"),
            "autolinks" =>  include_str!("style/autolinks.css"),
            "selflinks" => include_str!("style/selflinks.css"),
            "counters" => include_str!("style/counters.css"),
        };

        Spec {
            infile,
            lines,
            md_cli,
            extra_styles,
            reference_manager: ReferenceManager::new(),
            biblio_entry_manager: BiblioEntryManager::new(),
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
                        text,
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
        self.lines = markdown::comment::remove_comments(&self.lines);

        let (md_doc, lines) = metadata::parse_metadata(&self.lines);
        self.lines = lines;

        // [default metadata] < [document metadata] < [command-line metadata]
        let mut md = {
            let group = metadata::extract_group(&[&md_doc, &self.md_cli]);
            let status = metadata::extract_status(&[&md_doc, &self.md_cli]);
            let data =
                retrieve_boilerplate_with_info("defaults", group.as_deref(), status.as_deref());
            Metadata::from_json(data)
        };
        md.join(md_doc);
        md.join(self.md_cli.clone());

        md.compute_implicit_metadata();
        md.fill_macros(self);
        md.validate();
        self.md = md;

        let lines = markdown::parse(
            &self
                .lines
                .iter()
                .map(|l| l.text.clone())
                .collect::<Vec<String>>(),
            self.md.indent(),
        );

        self.html = lines.join("\n");
        boilerplate::add_header_footer(self);
        self.html = self.fix_text(&self.html);

        let dom = kuchiki::parse_html().one(self.html.clone());
        self.head = html::select_first(&dom, "head");
        self.body = html::select_first(&dom, "body");
        clean::correct_h1(&dom);
        self.dom = Some(dom);
    }

    fn process_document(&mut self) {
        boilerplate::load_containers(self);

        // Fill in sections.
        boilerplate::add_canonical_url(self);
        boilerplate::fill_spec_metadata_section(self);
        boilerplate::fill_copyright_section(self);
        boilerplate::fill_abstract_section(self);
        shorthand::transform_shortcuts(self);
        fix::canonicalize_shortcuts(self);

        // Handle links.
        dfn::process_dfns(self);
        link::process_auto_links(self);
        boilerplate::add_index_section(self);
        boilerplate::add_references_section(self);
        heading::process_headings(self);
        boilerplate::fill_toc_section(self);
        link::add_self_links(self);

        boilerplate::add_styles(self);
        boilerplate::add_scripts(self);

        // Clean the DOM before serialization.
        clean::clean_dom(self.dom());
    }

    pub fn finish(&mut self, outfile: Option<&str>) {
        let outfile = self.handle_outfile(outfile);
        let rendered = self.dom().to_string();
        fs::write(outfile, rendered).expect("unable to write file");
    }

    // Do several textual replacements with this spec.
    pub fn fix_text(&self, text: &str) -> String {
        if self.md.markup_shorthands.get("markdown") {
            let mut code_span_manager = CodeSpanManager::new(text.to_owned());
            code_span_manager.map_text_pieces(|text: &str| fix::replace_macros(text, &self.macros));
            code_span_manager.map_text_pieces(fix::fix_typography);
            code_span_manager.extract()
        } else {
            let mut text = fix::replace_macros(text, &self.macros);
            text = fix::fix_typography(&text);
            text
        }
    }

    fn handle_outfile(&self, outfile: Option<&str>) -> String {
        if let Some(outfile) = outfile {
            outfile.to_owned()
        } else {
            for extension in SOURCE_FILE_EXTENSIONS.iter() {
                if self.infile.ends_with(extension) {
                    return (&self.infile[..self.infile.len() - extension.len()]).to_string()
                        + ".html";
                }
            }
            "-".to_owned()
        }
    }

    pub fn dom(&self) -> &NodeRef {
        self.dom.as_ref().unwrap()
    }

    pub fn head(&self) -> &NodeRef {
        self.head.as_ref().unwrap()
    }

    pub fn body(&self) -> &NodeRef {
        self.body.as_ref().unwrap()
    }
}
