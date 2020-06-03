use kuchiki::traits::*;
use std::fs;
use std::path::Path;

use crate::clean;
use crate::metadata::metadata::Metadata;
use crate::spec::Spec;

#[test]
fn test_spec() {
    let src_path = Path::new("tests").join("metadata001.bs");
    let target_path = Path::new("tests").join("metadata001.html");

    let mut spec = Spec::new(src_path.to_str().unwrap(), Metadata::new());
    spec.preprocess();
    spec.render();

    match fs::read_to_string(target_path.to_str().unwrap()) {
        Ok(html) => {
            let expect_dom = kuchiki::parse_html().one(html);

            match clean::clean_html(expect_dom.to_string()) {
                // TODO: Print the first line that doesn't match.
                Ok(expect_html) => assert_eq!(spec.rendered, expect_html),
                _ => die!("Fail to render expect DOM tree."),
            }
        }
        _ => die!("Fail to read expect file"),
    }
}
