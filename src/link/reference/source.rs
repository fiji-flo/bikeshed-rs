use std::collections::HashMap;
use std::path::Path;

use super::Reference;
use crate::config;
use crate::util::reader;

#[derive(Debug, Default)]
pub struct ReferenceSource {
    base_path: String,
    // text => reference
    pub references: HashMap<String, Reference>,
}

impl ReferenceSource {
    pub fn new(base_path: &str) -> Self {
        ReferenceSource {
            base_path: base_path.to_owned(),
            ..Default::default()
        }
    }

    fn load(&mut self, group: &str) {
        let data_path = Path::new("spec-data")
            .join(&self.base_path)
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
            let _ = lines.next().unwrap().unwrap();
            let _ = lines.next().unwrap().unwrap();
            let _ = lines.next().unwrap().unwrap();
            let url = lines.next().unwrap().unwrap();
            let _ = lines.next().unwrap().unwrap();
            let _ = lines.next().unwrap().unwrap();

            loop {
                let line = lines.next().unwrap().unwrap();

                if line == "-" {
                    break;
                }
            }

            self.references.insert(
                key,
                Reference {
                    link_type,
                    spec: Some(spec),
                    url,
                },
            );
        }
    }

    pub fn fetch_reference(&mut self, link_type: &str, link_text: &str) -> Reference {
        if let Some(reference) = self.references.get(link_text) {
            if link_type == reference.link_type {
                return reference.to_owned();
            }
        }

        let group = config::generate_group_name(link_text);
        self.load(&group);

        let reference = self.references.get(link_text).unwrap().to_owned();
        assert_eq!(link_type, reference.link_type);
        reference
    }
}
