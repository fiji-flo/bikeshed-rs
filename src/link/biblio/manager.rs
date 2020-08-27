use super::source::{BiblioFormat, BiblioSource};
use super::Biblio;

#[derive(Debug, Default)]
pub struct BiblioManager {
    pub biblio_source: BiblioSource,
}

impl BiblioManager {
    pub fn new() -> Self {
        BiblioManager {
            biblio_source: BiblioSource::new("biblio"),
        }
    }

    pub fn get_biblio(&mut self, link_text: &str) -> Option<Biblio> {
        let link_text = link_text.to_lowercase();

        if let Some(biblio) = self.biblio_source.fetch_biblio(&link_text) {
            if biblio.biblio_format == BiblioFormat::Alias {}
            match biblio.biblio_format {
                BiblioFormat::Alias => return self.get_biblio(biblio.alias_of.as_ref().unwrap()),
                _ => return Some(biblio),
            }
        }

        None
    }
}
