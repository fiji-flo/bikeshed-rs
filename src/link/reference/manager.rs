use kuchiki::NodeRef;

use super::query::Query;
use super::source::{ReferenceSource, SourceKind};
use super::Reference;
use crate::config;
use crate::html;
use crate::metadata::Metadata;

#[derive(Debug, Default)]
pub struct ReferenceManager {
    pub local_reference_source: ReferenceSource,
    pub anchor_block_reference_source: ReferenceSource,
    pub external_reference_source: ReferenceSource,
    pub spec: Option<String>,
}

impl ReferenceManager {
    pub fn new() -> Self {
        ReferenceManager {
            local_reference_source: ReferenceSource::new(SourceKind::Local),
            anchor_block_reference_source: ReferenceSource::new(SourceKind::AnchorBlock),
            external_reference_source: ReferenceSource::new(SourceKind::External),
            ..Default::default()
        }
    }

    pub fn set_data(&mut self, md: &Metadata) {
        self.spec = Some(md.vshortname());
    }

    pub fn get_reference(&mut self, query: Query) -> Reference {
        let link_type = query.link_type;
        let link_text = query.link_text;
        let link_fors = query.link_fors;

        // Load local references.
        if let Ok(local_references) = self.local_reference_source.query_references(Query {
            link_type,
            link_text,
            status: None,
            link_fors,
        }) {
            return local_references[0].to_owned();
        }

        // Load anchor block references.
        if let Ok(anchor_block_references) =
            self.anchor_block_reference_source.query_references(Query {
                link_type,
                link_text,
                status: None,
                link_fors,
            })
        {
            return anchor_block_references[0].to_owned();
        }

        // Load external references.
        let external_references = self
            .external_reference_source
            .query_references(Query {
                link_type,
                link_text,
                status: Some("current"),
                link_fors,
            })
            .unwrap();

        external_references[0].to_owned()
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
                status: "local".to_owned(),
                url: format!("#{}", html::get_attr(&dfn_el, "id").unwrap()),
                link_fors,
            };

            self.local_reference_source
                .add_reference(link_text, reference);
        }
    }
}
