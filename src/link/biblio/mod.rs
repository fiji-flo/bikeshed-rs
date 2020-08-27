pub mod manager;
mod source;

use kuchiki::NodeRef;

use crate::html::{self, Attr};
use source::BiblioFormat;

#[derive(Debug, Default, Clone)]
pub struct Biblio {
    pub biblio_format: BiblioFormat,
    pub link_text: String,
    pub date: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub authors: Vec<String>,
    pub data: Option<String>,
    pub alias_of: Option<String>,
}

impl Biblio {
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

        match &self.url {
            Some(url) => dd_el.append(html::new_a(
                btreemap! {
                    "href" => url,
                },
                self.title.as_ref().unwrap(),
            )),
            None => dd_el.append(html::new_text(self.title.as_ref().unwrap())),
        };

        if let Some(ref date) = self.date {
            dd_el.append(html::new_text(format!(". {}", date)));
        }

        if let Some(ref status) = self.status {
            dd_el.append(html::new_text(format!(". {}", status)));
        }

        if let Some(ref url) = self.url {
            dd_el.append(html::new_text(". URL: "));

            dd_el.append(html::new_a(
                btreemap! {
                    "href" => url,
                },
                url,
            ));
        }

        dd_el
    }
}
