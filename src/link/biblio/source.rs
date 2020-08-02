use std::collections::HashMap;
use std::path::Path;

use super::BiblioEntry;
use crate::util::reader;

#[derive(Debug, Default)]
pub struct BiblioEntrySource {
    base_path: String,
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
        let mut lines = reader::read_lines(data_path).unwrap();

        while let Some(full_key) = lines.next() {
            let full_key = full_key.unwrap();
            let prefix = &full_key[0..1];
            let key = full_key[2..].trim_end();

            match prefix {
                "d" => {
                    let link_text = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let title = lines.next().unwrap().unwrap();
                    let url = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();
                    let _ = lines.next().unwrap().unwrap();

                    self.biblio_entries.insert(
                        key.to_owned(),
                        BiblioEntry {
                            link_text,
                            title,
                            url,
                        },
                    );

                    loop {
                        let curr_line = lines.next().unwrap().unwrap();
                        if curr_line == "-" {
                            break;
                        }
                    }
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

    pub fn fetch_biblio_entry(&mut self, key: &str) -> BiblioEntry {
        if let Some(biblio_entry) = self.biblio_entries.get(key) {
            return biblio_entry.to_owned();
        }

        let group = &key[..2];
        self.load(group);

        self.biblio_entries.get(key).unwrap().to_owned()
    }
}
