use std::collections::HashMap;
use std::path::Path;

use super::Reference;
use crate::util::reader;

#[derive(Debug, Default)]
pub struct ReferenceSource {
    base_path: String,
    // text => reference
    references: HashMap<String, Reference>,
}

impl ReferenceSource {
    pub fn new(base_path: &str) -> Self {
        ReferenceSource {
            base_path: base_path.to_owned(),
            ..Default::default()
        }
    }

    fn load(&mut self, group: &str) {
        let data_path = Path::new(&self.base_path).join(format!("anchors-{}.data", group));
        let mut lines = reader::read_lines(data_path).unwrap();

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
                let curr_line = lines.next().unwrap().unwrap();
                if curr_line == "-" {
                    break;
                }
            }

            self.references.insert(
                key,
                Reference {
                    link_type,
                    spec,
                    url,
                },
            );
        }
    }

    pub fn fetch_reference(&mut self, key: &str) -> Reference {
        if let Some(reference) = self.references.get(key) {
            return reference.to_owned();
        }

        let group = &key[..2];
        self.load(group);

        self.references.get(key).unwrap().to_owned()
    }
}
