use super::source::ReferenceSource;

#[derive(Debug, Default, Clone)]
pub struct Reference {
    pub link_type: String,
    pub spec: String,
    pub url: String,
}

#[derive(Debug, Default)]
pub struct ReferenceManager {
    pub reference_source: ReferenceSource,
}

impl ReferenceManager {
    pub fn new() -> Self {
        ReferenceManager {
            reference_source: ReferenceSource::new("spec-data"),
        }
    }

    pub fn get_reference(&mut self, link_text: &str) -> Reference {
        self.reference_source.fetch_reference(link_text)
    }
}
