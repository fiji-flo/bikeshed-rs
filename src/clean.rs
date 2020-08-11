use kuchiki::NodeRef;

use crate::config::DFN_SELECTOR;
use crate::html;

// If an <h1> was provided manually, use that element rather than whatever the boilerplate contains.
pub fn correct_h1(dom: &NodeRef) {
    let h1_els = html::select(dom, "h1").collect::<Vec<NodeRef>>();

    if h1_els.len() >= 2 {
        html::replace_node(&h1_els[0], &h1_els[1]);
    }
}

// Clean the DOM before serialization.
pub fn clean_dom(dom: &NodeRef) {
    // TODO: Traverse the DOM only once?

    // If a <dt> contains only a single paragraph, extract its content.
    for dt_el in html::select(dom, "dt[data-md]") {
        let child = match html::get_only_child(&dt_el) {
            Some(child) => child,
            None => continue,
        };

        let p_el = if html::get_tag(&child).unwrap() == "p" {
            child
        } else {
            continue;
        };

        html::copy_content(&p_el, &dt_el);
        p_el.detach();
    }

    // Allow Markdown-generated list to be surrounded by HTML list container.
    for list_el in html::select(dom, "ol, ul, dl") {
        let child_el = match html::get_only_child(&list_el) {
            Some(only_child) => only_child,
            None => {
                html::remove_attr(&list_el, "data-md");
                continue;
            }
        };

        if html::get_tag(&list_el).unwrap() == html::get_tag(&child_el).unwrap()
            && !html::has_attr(&list_el, "data-md")
            && html::has_attr(&child_el, "data-md")
        {
            html::copy_content(&child_el, &list_el);
            child_el.detach();
            continue;
        }

        html::remove_attr(&list_el, "data-md");
    }

    for dfn_el in html::select(dom, &DFN_SELECTOR) {
        // Check dfn type.
        match html::get_attr(&dfn_el, "data-dfn-type") {
            Some(dfn_type) => {
                if dfn_type != "property" {
                    continue;
                }
            }
            None => continue,
        };

        if !html::has_ancestor(&dfn_el, |el| match html::get_tag(el) {
            Some(tag) => tag == "pre",
            None => false,
        }) {
            html::add_class(&dfn_el, "css")
        }

        html::add_class(&dfn_el, "css");
    }

    for a_el in html::select(dom, "a") {
        // Check link type.
        match html::get_attr(&a_el, "data-link-type") {
            Some(dfn_type) => {
                if dfn_type != "property" {
                    continue;
                }
            }
            None => continue,
        };

        if !html::has_ancestor(&a_el, |el| match html::get_tag(el) {
            Some(tag) => tag == "pre",
            None => false,
        }) {
            html::add_class(&a_el, "css")
        }

        html::add_class(&a_el, "css");
    }
}
