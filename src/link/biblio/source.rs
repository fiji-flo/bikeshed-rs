use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::Biblio;
use crate::config;
use crate::util::reader;

#[derive(Debug, Clone, PartialEq)]
pub enum BiblioFormat {
    Dict,
    Str,
    Alias,
}

impl Default for BiblioFormat {
    fn default() -> Self {
        BiblioFormat::Dict
    }
}

#[derive(Debug, Default)]
pub struct BiblioSource {
    base_path: String,
    loaded_groups: HashSet<String>,
    // text => biblios
    biblios: HashMap<String, Biblio>,
}

impl BiblioSource {
    pub fn new(base_path: &str) -> Self {
        BiblioSource {
            base_path: base_path.to_owned(),
            ..Default::default()
        }
    }

    fn load(&mut self, group: &str) {
        let data_path = Path::new("spec-data")
            .join(&self.base_path)
            .join(format!("biblio-{}.data", group));

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

            let biblio = match prefix {
                "d" => {
                    let link_text = lines.next().unwrap().unwrap();
                    let date = lines.next().unwrap().unwrap();
                    let status = lines.next().unwrap().unwrap();
                    let title = lines.next().unwrap().unwrap();
                    let url = lines.next().unwrap().unwrap();
                    lines.next(); // current url
                    lines.next(); // obsoleted by
                    lines.next(); // other
                    lines.next(); // el at or not

                    let mut authors = Vec::new();

                    loop {
                        let line = lines.next().unwrap().unwrap();

                        if line == "-" {
                            break;
                        }

                        authors.push(line);
                    }

                    Biblio {
                        biblio_format: BiblioFormat::Dict,
                        link_text,
                        date: Some(date),
                        status: Some(status),
                        title: Some(title),
                        url: Some(url),
                        authors,
                        ..Default::default()
                    }
                }
                "s" => {
                    let link_text = lines.next().unwrap().unwrap();
                    let data = lines.next().unwrap().unwrap();
                    lines.next();

                    Biblio {
                        biblio_format: BiblioFormat::Str,
                        link_text,
                        data: Some(data),
                        ..Default::default()
                    }
                }
                "a" => {
                    let link_text = lines.next().unwrap().unwrap();
                    let alias_of = lines.next().unwrap().unwrap();
                    lines.next();

                    Biblio {
                        biblio_format: BiblioFormat::Alias,
                        link_text,
                        alias_of: Some(alias_of),
                        ..Default::default()
                    }
                }
                _ => die!("Unknown biblio prefix: {}.", prefix),
            };

            self.biblios.insert(key.to_owned(), biblio);
        }
    }

    pub fn fetch_biblio(&mut self, key: &str) -> Option<Biblio> {
        if let Some(biblio) = self.biblios.get(key) {
            return Some(biblio.to_owned());
        }

        let group = config::generate_group_name(key);

        if self.loaded_groups.contains(group.as_str()) {
            return None;
        }

        self.load(&group);
        self.loaded_groups.insert(group);

        self.biblios.get(key).map(ToOwned::to_owned)
    }
}
