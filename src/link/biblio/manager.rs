use super::source::{BiblioEntrySource, BiblioFormat};
use super::BiblioEntry;

#[derive(Debug, Default)]
pub struct BiblioEntryManager {
    pub biblio_entry_source: BiblioEntrySource,
}

impl BiblioEntryManager {
    pub fn new() -> Self {
        BiblioEntryManager {
            biblio_entry_source: BiblioEntrySource::new("biblio"),
        }
    }

    pub fn get_biblio_entry(&mut self, link_text: &str) -> Option<BiblioEntry> {
        let link_text = link_text.to_lowercase();

        if let Some(biblio_entry) = self.biblio_entry_source.fetch_biblio_entry(&link_text) {
            if biblio_entry.biblio_format == BiblioFormat::Alias {}
            match biblio_entry.biblio_format {
                BiblioFormat::Alias => {
                    return self.get_biblio_entry(biblio_entry.alias_of.as_ref().unwrap())
                }
                _ => return Some(biblio_entry),
            }
        }

        None
    }
}
