use kuchiki::{NodeDataRef, NodeRef};

use crate::html;

pub fn correct_h1(document: &NodeRef) {
    // If an <h1> was provided manually, use that element rather than whatever the boilerplate contains.
    if let Ok(h1s) = document.select("h1") {
        let h1s = h1s.collect::<Vec<NodeDataRef<_>>>();
        if h1s.len() >= 2 {
            html::node::replace_node(h1s[0].as_node(), h1s[1].as_node());
        }
    }
}
