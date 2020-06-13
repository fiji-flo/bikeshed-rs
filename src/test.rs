use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::fs;
use std::path::Path;

use crate::html;
use crate::metadata::metadata::Metadata;
use crate::spec::Spec;

fn trim_text_node(el: NodeRef) -> NodeRef {
    match el.as_text() {
        Some(text) => html::node::new_text(text.clone().into_inner().trim()),
        None => el.clone(),
    }
}

fn is_valid_node(el: &NodeRef) -> bool {
    match el.as_text() {
        Some(text) => !text.borrow().is_empty(),
        None => true,
    }
}

// Compare DOM trees recursively.
fn is_equal(lhs: &NodeRef, rhs: &NodeRef) -> bool {
    if lhs.data() != rhs.data() {
        return false;
    }

    // handle empty text nodes
    let lhs_children = lhs
        .children()
        .map(trim_text_node)
        .filter(is_valid_node)
        .collect::<Vec<NodeRef>>();
    let rhs_children = rhs
        .children()
        .map(trim_text_node)
        .filter(is_valid_node)
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
