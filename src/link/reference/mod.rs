pub mod manager;
mod source;

#[derive(Debug, Default, Clone)]
pub struct Reference {
    pub link_type: String,
    pub spec: Option<String>,
    pub url: String,
    pub link_fors: Vec<String>,
}
