use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::query::Query;
use super::Reference;
use crate::config;
use crate::util::reader;

#[derive(Debug, PartialEq)]
pub enum SourceKind {
    Local,
    AnchorBlock,
    External,
}

impl Default for SourceKind {
    fn default() -> Self {
        SourceKind::Local
    }
}

#[derive(Debug, Default)]
pub struct ReferenceSource {
    source_kind: SourceKind,
    loaded_groups: HashSet<String>,
    // text => references
    references: HashMap<String, Vec<Reference>>,
}

#[derive(Debug)]
pub enum QueryError {
    Text,
    LinkType,
    Status,
    For,
}

impl ReferenceSource {
    pub fn new(source_kind: SourceKind) -> Self {
        ReferenceSource {
            source_kind,
            ..Default::default()
        }
    }

    pub fn query_references(&mut self, query: Query) -> Result<Vec<Reference>, QueryError> {
        // Filter references by link text.
        let mut references = self.fetch_references(query.link_text);

        if references.is_empty() {
            return Err(QueryError::Text);
        }

        // Filter references by link type.
        references = references
            .iter()
            .filter(|reference| reference.link_type == query.link_type)
            .map(ToOwned::to_owned)
            .collect();

        if references.is_empty() {
            return Err(QueryError::LinkType);
        }

        // Filter references by status.
        if let Some(status) = query.status {
            references = references
                .iter()
                .filter(|reference| reference.status == status)
                .map(ToOwned::to_owned)
                .collect();

            if references.is_empty() {
                return Err(QueryError::Status);
            }
        }

        // Filter references by link for values.
        fn match_link_fors(target_link_fors: &[String], test_link_fors: &[String]) -> bool {
            if test_link_fors.len() == 1 && test_link_fors[0] == "/" {
                target_link_fors.is_empty()
            } else {
                test_link_fors
                    .iter()
                    .any(|test_link_for| target_link_fors.contains(test_link_for))
            }
        }

        fn filter_by_for_vals(
            references: &[Reference],
            test_link_fors: &[String],
        ) -> Vec<Reference> {
            references
                .iter()
                .filter(|reference| match_link_fors(&reference.link_fors, test_link_fors))
                .map(ToOwned::to_owned)
                .collect()
        }

        if let Some(link_fors) = query.link_fors {
            references = filter_by_for_vals(&references, link_fors);
        }

        if references.is_empty() {
            return Err(QueryError::For);
        }

        Ok(references)
    }

    pub fn add_reference(&mut self, link_text: String, reference: Reference) {
        self.references
            .entry(link_text)
            .or_default()
            .push(reference);
    }

    fn fetch_references(&mut self, link_text: &str) -> Vec<Reference> {
        if let Some(references) = self.references.get(link_text) {
            return references.to_owned();
        }

        if self.source_kind != SourceKind::External {
            return Vec::new();
        }

        let group = config::generate_group_name(link_text);

        if self.loaded_groups.contains(&group) {
            return Vec::new();
        }

        self.load_spec_data(&group);
        self.loaded_groups.insert(group);

        match self.references.get(link_text) {
            Some(references) => references.to_owned(),
            None => Vec::new(),
        }
    }

    fn load_spec_data(&mut self, group: &str) {
        let data_path = Path::new("spec-data")
            .join("anchors")
            .join(format!("anchors-{}.data", group));

        let mut lines = match reader::read_lines(&data_path) {
            Ok(lines) => lines,
            _ => die!(
                "Fail to load reference data file: \"{}\".",
                data_path.to_str().unwrap()
            ),
        };

        while let Some(key) = lines.next() {
            let key = key.unwrap();
            let link_type = lines.next().unwrap().unwrap();
            let spec = lines.next().unwrap().unwrap();
            lines.next(); // shortname
            lines.next(); // level
            let status = lines.next().unwrap().unwrap();
            let url = lines.next().unwrap().unwrap();
            lines.next(); // export
            lines.next(); // normative

            let mut link_fors = Vec::new();

            loop {
                let line = lines.next().unwrap().unwrap();

                if line == "-" {
                    break;
                }

                link_fors.push(line);
            }

            let reference = Reference {
                link_type,
                spec: Some(spec),
                status,
                url,
                link_fors,
            };

            self.add_reference(key, reference);
        }
    }
}
