use kuchiki::NodeRef;

use crate::html;
use crate::spec::Spec;

pub fn process_headings(doc: &mut Spec) {
    // TODO:
    // 1. [all] or [doc-only]?
    // 2. [settled] or not?

    if let Ok(els) = doc.dom().select("h2, h3, h4, h5, h6") {
        for el in els {
            html::node::add_class(el.as_node(), "heading");
            html::node::add_class(el.as_node(), "settled");
            wrap_heading_contents(el.as_node());
        }
    }
}

// Insert the content of heading into a <span class='content'> element.
fn wrap_heading_contents(heading_el: &NodeRef) {
    let content_el = html::node::new_element(
        "span",
        btreemap! {
            "class" => "content",
        },
    );

    for child in heading_el.children() {
        content_el.append(child);
    }

    heading_el.append(content_el);
}
