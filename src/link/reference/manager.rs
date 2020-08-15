use kuchiki::NodeRef;
use std::collections::HashMap;

use super::source::ReferenceSource;
use super::Reference;
use crate::config;
use crate::html;
use crate::metadata::Metadata;

#[derive(Debug, Default)]
pub struct ReferenceManager {
    pub reference_source: ReferenceSource,
    pub local_references: HashMap<String, Reference>,
    pub spec: Option<String>,
}

impl ReferenceManager {
    pub fn new() -> Self {
        ReferenceManager {
            reference_source: ReferenceSource::new("anchors"),
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, md: &Metadata) {
        self.spec = Some(md.vshortname());
    }

    pub fn get_reference(&mut self, link_type: &str, link_text: &str) -> Reference {
        match self.fetch_local_reference(link_type, link_text) {
            Some(local_reference) => local_reference,
            None => self.reference_source.fetch_reference(link_type, link_text),
        }
    }

    fn fetch_local_reference(&self, link_type: &str, link_text: &str) -> Option<Reference> {
        self.local_references
            .get(link_text)
            .filter(|reference| link_type == reference.link_type)
            .map(ToOwned::to_owned)
    }

    pub fn add_local_dfns(&mut self, dfn_els: &[NodeRef]) {
        for dfn_el in dfn_els {
            let link_text = html::get_text_content(&dfn_el);
            let link_type = html::closest_attr_in(&dfn_el, "data-dfn-type").unwrap();

            let link_fors = match html::closest_attr_in(&dfn_el, "data-dfn-for") {
                Some(dfn_for) => config::split_for_vals(&dfn_for),
                None => Vec::new(),
            };

            let reference = Reference {
                link_type,
                spec: self.spec.to_owned(),
                url: format!("#{}", html::get_attr(&dfn_el, "id").unwrap()),
                link_fors,
            };

            self.local_references.insert(link_text, reference);
        }
    }
}
