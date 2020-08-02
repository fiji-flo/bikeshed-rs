pub mod manager;
mod source;

use kuchiki::NodeRef;

use crate::html::{self, Attr};

#[derive(Debug, Default, Clone)]
pub struct BiblioEntry {
    pub link_text: String,
    pub date: String,
    pub status: String,
    pub title: String,
    pub url: String,
    pub authors: Vec<String>,
}

impl BiblioEntry {
    pub fn to_node(&self) -> NodeRef {
        let dd_el = html::new_element("dd", None::<Attr>);

        let authors_text = if self.authors.len() == 1 {
            format!("{}. ", self.authors[0])
        } else if self.authors.len() < 4 {
            format!("{}. ", self.authors.join("; "))
        } else {
            format!("{}; et al. ", self.authors[0])
        };
        dd_el.append(html::new_text(authors_text));

        dd_el.append(html::new_a(
            btreemap! {
                "href" => &self.url,
            },
            &self.title,
        ));

        dd_el.append(html::new_text(format!(
            ". {}. {}. URL: ",
            self.date, self.status
        )));

        dd_el.append(html::new_a(
            btreemap! {
                "href" => &self.url,
            },
            &self.url,
        ));

        dd_el
    }
}
