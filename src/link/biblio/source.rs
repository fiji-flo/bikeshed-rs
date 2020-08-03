use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::BiblioEntry;
use crate::config;
use crate::util::reader;

#[derive(Debug, Default)]
pub struct BiblioEntrySource {
    base_path: String,
    loaded_groups: HashSet<String>,
    // text => biblio entry
    biblio_entries: HashMap<String, BiblioEntry>,
}

impl BiblioEntrySource {
    pub fn new(base_path: &str) -> Self {
        BiblioEntrySource {
            base_path: base_path.to_owned(),
            ..Default::default()
        }
    }

    fn load(&mut self, group: &str) {
        let data_path = Path::new(&self.base_path).join(format!("biblio-{}.data", group));

        let mut lines = match reader::read_lines(&data_path) {
            Ok(lines) => lines,
            _ => die!(
                "Fail to load biblio data file: \"{}\".",
                data_path.to_str().unwrap()
            ),
        };

        while let Some(full_key) = lines.next() {
            let full_key = full_key.unwrap();
            let prefix = &full_key[0..1];
            let key = full_key[2..].trim_end();

            match prefix {
                "d" => {
                    let link_text = lines.next().unwrap().unwrap();
                    let date = lines.next().unwrap().unwrap();
                    let status = lines.next().unwrap().unwrap();
                    let title = lines.next().unwrap().unwrap();
                    let url = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();

                    let mut authors = Vec::new();

                    loop {
                        let line = lines.next().unwrap().unwrap();

                        if line == "-" {
                            break;
                        }

                        authors.push(line);
                    }

                    self.biblio_entries.insert(
                        key.to_owned(),
                        BiblioEntry {
                            link_text,
                            date,
                            status,
                            title,
                            url,
                            authors,
                        },
                    );
                }
                "a" | "s" => {
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                }
                _ => die!("Unknown biblio prefix: {}.", prefix),
            }
        }
    }

    pub fn fetch_biblio_entry(&mut self, key: &str) -> Option<BiblioEntry> {
        if let Some(biblio_entry) = self.biblio_entries.get(key) {
            return Some(biblio_entry.to_owned());
        }

        let group = config::generate_group_name(key);

        if self.loaded_groups.contains(group.as_str()) {
            return None;
        }

        self.load(&group);
        self.loaded_groups.insert(group);

        self.biblio_entries.get(key).map(ToOwned::to_owned)
    }
}
