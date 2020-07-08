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

// Clean the DOM before serialization.
pub fn clean_dom(dom: &NodeRef) {
    // TODO: Traverse the DOM only once?

    // If a <dt> contains only a single paragraph, extract its content.
    if let Ok(dt_els) = dom.select("dt[data-md]") {
        for dt_el in dt_els {
            let dt_el = dt_el.as_node();

            let child = match html::get_only_child(dt_el) {
                Some(child) => child,
                None => continue,
            };

            let p_el = if html::get_tag(&child).unwrap() == "p" {
                child
            } else {
                continue;
            };

            html::copy_content(&p_el, dt_el);
            p_el.detach();
        }
    }

    // Allow Markdown-generated list to be surrounded by HTML list container.
    if let Ok(list_els) = dom.select("ol, ul, dl") {
        for list_el in list_els {
            let list_el = list_el.as_node();

            let child_el = match html::get_only_child(list_el) {
                Some(only_child) => only_child,
                None => {
                    html::remove_attr(list_el, "data-md");
                    continue;
                }
            };

            if html::get_tag(list_el).unwrap() == html::get_tag(&child_el).unwrap()
                && !html::has_attr(list_el, "data-md")
                && html::has_attr(&child_el, "data-md")
            {
                html::copy_content(&child_el, list_el);
                child_el.detach();
                continue;
            }

            html::remove_attr(list_el, "data-md");
        }
    }
}
