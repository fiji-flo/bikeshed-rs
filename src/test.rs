use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::fs;
use std::path::Path;

use crate::html;
use crate::metadata::Metadata;
use crate::spec::Spec;

fn merge_text_nodes(text_els: &[NodeRef]) -> NodeRef {
    let mut texts = Vec::new();

    for text_el in text_els {
        let text = html::unwrap_text_node(text_el);
        texts.push(text);
    }

    html::new_text(texts.join(""))
}

fn preprocess_text_nodes(els: &[NodeRef]) -> Vec<NodeRef> {
    let mut new_els = Vec::new();
    let mut curr = 0;

    while curr < els.len() {
        if !html::is_text_node(&els[curr]) {
            new_els.push(els[curr].clone());
            curr += 1;
            continue;
        }

        let mut text_els = vec![els[curr].clone()];

        while curr + 1 < els.len() && html::is_text_node(&els[curr + 1]) {
            curr += 1;
            text_els.push(els[curr].clone());
        }

        new_els.push(merge_text_nodes(&text_els));
        curr += 1;
    }

    new_els
}

fn trim_text_node(el: &NodeRef) -> NodeRef {
    match el.as_text() {
        Some(text) => html::new_text(text.clone().into_inner().trim()),
        None => el.clone(),
    }
}

fn is_valid_node(el: &NodeRef) -> bool {
    match el.as_text() {
        Some(text) => !text.borrow().is_empty(),
        None => true,
    }
}

enum CompareError {
    Data(NodePair),
    Number(NodePair),
}

struct NodePair {
    result: NodeRef,
    expect: NodeRef,
}

// Compare DOM trees recursively.
fn is_equal(lhs: &NodeRef, rhs: &NodeRef) -> Result<(), CompareError> {
    if lhs.data() != rhs.data() {
        return Err(CompareError::Data(NodePair {
            result: lhs.clone(),
            expect: rhs.clone(),
        }));
    }

    let mut lhs_children = lhs.children().collect::<Vec<NodeRef>>();
    lhs_children = preprocess_text_nodes(&lhs_children);
    lhs_children = lhs_children
        .iter()
        .map(trim_text_node)
        .filter(is_valid_node)
        .collect::<Vec<NodeRef>>();

    let mut rhs_children = rhs.children().collect::<Vec<NodeRef>>();
    rhs_children = preprocess_text_nodes(&rhs_children);
    rhs_children = rhs_children
        .iter()
        .map(trim_text_node)
        .filter(is_valid_node)
        .collect::<Vec<NodeRef>>();

    if lhs_children.len() != rhs_children.len() {
        return Err(CompareError::Number(NodePair {
            result: lhs.clone(),
            expect: rhs.clone(),
        }));
    }

    for (lc, rc) in lhs_children.iter().zip(rhs_children.iter()) {
        if let Err(err) = is_equal(lc, rc) {
            return Err(err);
        }
    }

    Ok(())
}

#[test]
fn test_spec() {
    // TODO: Use all files.
    let names = [
        // metadata
        "metadata001",
        "metadata002",
        "metadata003",
        "metadata004",
        "metadata005",
        "metadata006",
        "metadata007",
        "metadata008",
        "metadata009",
        "metadata010",
        "metadata011",
        "metadata012",
        "metadata013",
        "metadata014",
        "metadata015",
        "metadata016",
        // markdown
        "markdown001",
        "markdown002",
        "markdown003",
        "markdown004",
        "markdown006",
        "markdown007",
        "markdown008",
        "markdown009",
        "markdown010",
        "markdown011",
        "markdown012",
        "markdown013",
        // links
        "links001",
        "links002",
        "links003",
        "links004",
    ];

    for name in names.iter() {
        let src_path = Path::new("tests").join(format!("{}.bs", name));
        let target_path = Path::new("tests").join(format!("{}.html", name));

        let mut spec = Spec::new(src_path.to_str().unwrap(), Metadata::new());
        spec.preprocess();

        match fs::read_to_string(target_path.to_str().unwrap()) {
            Ok(html) => {
                let expect_dom = kuchiki::parse_html().one(html);

                if let Err(err) = is_equal(spec.dom(), &expect_dom) {
                    match err {
                        CompareError::Data(pair) => eprintln!(
                            "[{}] [Wrong Data]\nExpect:\n{}\n\nFound:\n{}",
                            name,
                            pair.expect.to_string(),
                            pair.result.to_string()
                        ),
                        CompareError::Number(pair) => eprintln!(
                            "[{}] [Wrong Children Number]\nExpect:\n{}\n\nFound:\n{}",
                            name,
                            pair.expect.to_string(),
                            pair.result.to_string()
                        ),
                    }
                    panic!();
                }
            }
            _ => die!("Fail to read expect file"),
        }
    }
}
