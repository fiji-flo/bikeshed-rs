use super::source::BiblioEntrySource;
use super::BiblioEntry;

#[derive(Debug, Default)]
pub struct BiblioEntryManager {
    pub biblio_entry_source: BiblioEntrySource,
}

impl BiblioEntryManager {
    pub fn new() -> Self {
        BiblioEntryManager {
            biblio_entry_source: BiblioEntrySource::new("spec-data"),
        }
    }

    pub fn get_biblio_entry(&mut self, spec: &str) -> BiblioEntry {
        self.biblio_entry_source.fetch_biblio_entry(spec)
    }
}
