use kuchiki::NodeRef;
use std::collections::HashMap;

use crate::html;

#[derive(Debug, Default)]
struct Reference {
    dfn_type: String,
    url: String,
}

#[derive(Debug, Default)]
pub struct ReferenceManager {
    local_references: HashMap<String, Vec<Reference>>,
}

impl ReferenceManager {
    pub fn add_local_dfns(&mut self, dfn_els: &[NodeRef]) {
        for dfn_el in dfn_els {
            let dfn_type = html::get_attr(dfn_el, "data-dfn-type").unwrap();
            let dfn_text = html::get_text_content(dfn_el);
            let id = html::get_attr(dfn_el, "id").unwrap();

            let references = self
                .local_references
                .entry(dfn_text)
                .or_insert_with(Vec::new);

            references.push(Reference {
                dfn_type,
                url: format!("#{}", id),
            });
        }
    }
}
