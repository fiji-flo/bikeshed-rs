use kuchiki::NodeDataRef;

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
            }
        }
    }
}
