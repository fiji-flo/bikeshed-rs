use kuchiki::NodeRef;
use std::collections::HashMap;

use super::biblio::BiblioEntry;
use crate::config;
use crate::html;

#[derive(Debug, Default, Clone)]
pub struct Reference {
    pub link_type: String,
    pub url: String,
    pub spec: String,
}

#[derive(Debug, Default)]
pub struct ReferenceManager {
    local_references: HashMap<String, Vec<Reference>>,
}

impl ReferenceManager {
    pub fn add_local_dfns(&mut self, dfn_els: &[NodeRef]) {
        for dfn_el in dfn_els {
            let link_type = html::get_attr(dfn_el, "data-dfn-type").unwrap();
            let link_text = html::get_text_content(dfn_el);
            let name = config::generate_name(&link_text);

            let references = self.local_references.entry(link_text).or_default();

            references.push(Reference {
                link_type,
                url: format!("https://drafts.csswg.org/css-flexbox-1/#{}", name),
                spec: "css-flexbox-1".to_owned(),
            });
        }
    }

    pub fn get_reference(&self, link_text: &str) -> &Reference {
        &self.local_references.get(link_text).unwrap()[0]
    }

    pub fn get_biblio_entry(&self, spec: &str) -> BiblioEntry {
        BiblioEntry {
            link_text: spec.to_owned(),
            title: "CSS Flexible Box Layout Module Level 1".to_owned(),
            url: format!("https://www.w3.org/TR/{}/", spec),
        }
    }
}
