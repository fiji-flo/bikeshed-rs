#[derive(Debug)]
pub struct Query<'a> {
    pub link_type: &'a str,
    pub link_text: &'a str,
    pub status: Option<&'a str>,
}
