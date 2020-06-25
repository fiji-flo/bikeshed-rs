use kuchiki::{NodeDataRef, NodeRef};

use crate::html;

// If an <h1> was provided manually, use that element rather than whatever the boilerplate contains.
pub fn correct_h1(dom: &NodeRef) {
    if let Ok(h1_els) = dom.select("h1") {
        let h1_els = h1_els.collect::<Vec<NodeDataRef<_>>>();

        if h1_els.len() >= 2 {
            html::replace_node(h1_els[0].as_node(), h1_els[1].as_node());
        }
    }
}

// Clean DOM before serialization.
pub fn clean_dom(dom: &NodeRef) {
    // If a <dt> contains only a single paragraph, extract its content.
    if let Ok(dt_els) = dom.select("dt[data-md]") {
        for dt_el in dt_els {
            let child = match html::get_only_child(dt_el.as_node()) {
                Some(child) => child,
                None => continue,
            };

            let p_el = if html::get_tag(&child).unwrap() == "p" {
                child
            } else {
                continue;
            };

            html::copy_content(&p_el, dt_el.as_node());
            p_el.detach();
        }
    }
}
