use kuchiki::{NodeData, NodeDataRef, NodeRef};
use markup5ever::LocalName;

use crate::html;
use crate::spec::Spec;

pub fn process_headings(doc: &mut Spec) {
    // TODO:
    // 1. [all] or [doc-only]?
    // 2. [settled] or not?
    let dom = match doc.dom {
        Some(ref dom) => dom,
        None => return,
    };

    let headings = match dom.select("h2, h3, h4, h5, h6") {
        Ok(headings) => {
            // TODO: Find a better way to filter headings.
            headings
                .filter(|h| {
                    // [ignore headings in boilerplate]
                    // [just for testing]
                    if let NodeData::Element(h_data) = h.as_node().data() {
                        let ref mut attributes = h_data.attributes.borrow();

                        if let Some(id) = attributes.get(LocalName::from("id")) {
                            if id == "contents" {
                                return false;
                            }
                        }
                    }
                    true
                })
                .collect::<Vec<NodeDataRef<_>>>()
        }
        Err(_) => return,
    };

    for heading in headings {
        html::node::add_class(heading.as_node(), "heading");
        html::node::add_class(heading.as_node(), "settled");
        reset_heading(heading.as_node());
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
