use kuchiki::traits::*;
use reqwest;
use serde_json::{self, Value};
use std::fs;
use std::path::Path;

use crate::metadata::metadata::Metadata;
use crate::spec::Spec;

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

#[test]
fn test_spec() {
    let src_path = Path::new("tests").join("metadata001.bs");
    let target_path = Path::new("tests").join("metadata001.html");

    let mut spec = Spec::new(src_path.to_str().unwrap(), Metadata::new());
    spec.preprocess();
    spec.render();

    let spec_html = match clean_html(spec.rendered) {
        Ok(expect_html) => expect_html,
        _ => die!("Fail to clean HTML of spec."),
    };

    match fs::read_to_string(target_path.to_str().unwrap()) {
        Ok(html) => {
            let expect_dom = kuchiki::parse_html().one(html);

            let expect_html = match clean_html(expect_dom.to_string()) {
                Ok(expect_html) => expect_html,
                _ => die!("Fail to clean expected HTML."),
            };

            // TODO: Print the first line that doesn't match.
            assert_eq!(spec_html, expect_html)
        }
        _ => die!("Fail to read expect file"),
    }
}
