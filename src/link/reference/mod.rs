pub mod manager;
pub mod query;
mod source;

#[derive(Debug, Clone)]
pub struct Reference {
    pub link_type: String,
    pub spec: Option<String>,
    pub status: String,
    pub url: String,
    pub link_fors: Vec<String>,
}
