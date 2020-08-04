use super::source::ReferenceSource;
use super::Reference;

#[derive(Debug, Default)]
pub struct ReferenceManager {
    pub reference_source: ReferenceSource,
}

impl ReferenceManager {
    pub fn new() -> Self {
        ReferenceManager {
            reference_source: ReferenceSource::new("anchors"),
        }
    }

    pub fn get_reference(&mut self, link_text: &str) -> Reference {
        self.reference_source.fetch_reference(link_text)
    }
}
