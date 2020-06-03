use kuchiki::{NodeDataRef, NodeRef};

use crate::html;
use crate::spec::Spec;

pub fn process_headings(doc: &mut Spec) {
    // TODO:
    // 1. [all] or [doc-only]?
    // 2. [settled] or not?
    if let Some(ref dom) = doc.dom {
        if let Ok(headings) = dom.select("h2, h3, h4, h5, h6") {
            let headings = headings.collect::<Vec<NodeDataRef<_>>>();

            for heading in headings {
                html::node::add_class(heading.as_node(), "heading");
                html::node::add_class(heading.as_node(), "settled");
                reset_heading(heading.as_node());
            }
        }
    }
}

// Insert the content of heading into a <span class='content'> element.
fn reset_heading(heading_el: &NodeRef) {
    let content_el = html::node::new_element(
        "span",
        btreemap! {
            "class" => "content".to_owned(),
        },
    );

    for child in heading_el.children() {
        content_el.append(child);
    }

    heading_el.append(content_el);
}
