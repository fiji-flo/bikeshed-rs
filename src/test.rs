use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::fs;
use std::path::Path;

use crate::metadata::metadata::Metadata;
use crate::spec::Spec;

// Compare DOM trees recursively.
fn is_equal(lhs: &NodeRef, rhs: &NodeRef) -> bool {
    if lhs.data() != rhs.data() {
        return false;
    }

    // TODO: Do it more accurately.
    // remove text nodes
    let lhs_children = lhs
        .children()
        .filter(|lc| lc.as_text().is_none())
        .collect::<Vec<NodeRef>>();
    let rhs_children = rhs
        .children()
        .filter(|rc| rc.as_text().is_none())
        .collect::<Vec<NodeRef>>();

    if lhs_children.len() != rhs_children.len() {
        return false;
    }

    for (lc, rc) in lhs_children.iter().zip(rhs_children.iter()) {
        if !is_equal(lc, rc) {
            return false;
        }
    }

    true
}

#[test]
fn test_spec() {
    let src_path = Path::new("tests").join("metadata001.bs");
    let target_path = Path::new("tests").join("metadata001.html");

    let mut spec = Spec::new(src_path.to_str().unwrap(), Metadata::new());
    spec.preprocess();

    match fs::read_to_string(target_path.to_str().unwrap()) {
        Ok(html) => {
            let expect_dom = kuchiki::parse_html().one(html);
            assert!(is_equal(spec.dom(), &expect_dom));
        }
        _ => die!("Fail to read expect file"),
    }
}
