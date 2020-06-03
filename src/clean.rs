use kuchiki::{NodeDataRef, NodeRef};
use reqwest;
use serde_json::{self, Value};

use crate::html;

pub fn correct_h1(dom: &NodeRef) {
    // If an <h1> was provided manually, use that element rather than whatever the boilerplate contains.
    if let Ok(h1s) = dom.select("h1") {
        let h1s = h1s.collect::<Vec<NodeDataRef<_>>>();
        if h1s.len() >= 2 {
            html::node::replace_node(h1s[0].as_node(), h1s[1].as_node());
        }
    }
}

pub fn clean_html(code: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let res = client
        .post("https://www.10bestdesign.com/dirtymarkup/api/html")
        .form(&[("code", code), ("indent", "2".to_owned())])
        .send()?
        .text()?;

    let val: Value = serde_json::from_str(&res)?;
    let cleaned = val["clean"].as_str().unwrap();

    Ok(cleaned.to_owned())
}
