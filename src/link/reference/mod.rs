pub mod manager;
mod source;

#[derive(Debug, Default, Clone)]
pub struct Reference {
    pub link_type: String,
    pub spec: String,
    pub url: String,
}
