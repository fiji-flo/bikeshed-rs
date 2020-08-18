mod replacer;

use kuchiki::NodeRef;
use regex::{Captures, Regex};

use crate::html;
use crate::spec::Spec;
use crate::util::boolset::BoolSet;
use replacer::*;

pub fn transform_shortcuts(doc: &Spec) {
    transform_node(doc.body(), &doc.md.markup_shorthands);
}

fn transform_node(el: &NodeRef, markup_shorthands: &BoolSet<String>) {
    let mut new_children = Vec::new();

    for child in el.children() {
        if html::is_text_node(&child) {
            let new_nodes = transform_text_node(&child, markup_shorthands);
            child.detach();
            new_children.extend(new_nodes);
        } else {
            if html::get_tag(&child).unwrap() != "code" {
                transform_node(&child, markup_shorthands);
            }
            new_children.push(child);
        }
    }

    for new_child in new_children {
        el.append(new_child);
    }
}

fn transform_text_node(text_el: &NodeRef, markup_shorthands: &BoolSet<String>) -> Vec<NodeRef> {
    let mut text_els = vec![text_el.clone()];

    if markup_shorthands.get("biblio") {
        text_els = process_text_nodes(&text_els, &BIBLIO_LINK_REG, biblio_link_replacer);
    }

    if markup_shorthands.get("algorithm") {
        text_els = process_text_nodes(&text_els, &VAR_REG, var_replacer);
    }

    if markup_shorthands.get("markdown") {
        text_els = process_text_nodes(&text_els, &INLINE_LINK_REG, inline_link_replacer);
        text_els = process_text_nodes(&text_els, &STRONG_REG, strong_replacer);
        text_els = process_text_nodes(&text_els, &EMPHASIS_REG, emphasis_replacer);
        text_els = process_text_nodes(&text_els, &ESCAPED_ASTERISK_REG, escaped_asterisk_replacer);
    }

    text_els
}

// We only care about the text nodes.
fn process_text_nodes(
    els: &[NodeRef],
    reg: &Regex,
    replacer: impl Fn(&Captures) -> Vec<NodeRef>,
) -> Vec<NodeRef> {
    let mut new_els = Vec::new();

    for el in els {
        if html::is_text_node(el) {
            new_els.extend(replace_all(el, reg, &replacer));
        } else {
            new_els.push(el.clone());
        }
    }

    new_els
}

fn replace_all(
    text_el: &NodeRef,
    reg: &Regex,
    replacer: &impl Fn(&Captures) -> Vec<NodeRef>,
) -> Vec<NodeRef> {
    let text = html::unwrap_text_node(text_el);
    let mut els = Vec::new();
    let mut last_end = 0;

    for mat in reg.find_iter(&text) {
        let start = mat.start();
        let end = mat.end();

        els.push(html::new_text(&text[last_end..start]));

        let caps = reg.captures(&text[start..end]).unwrap();
        els.extend(replacer(&caps));

        last_end = end;
    }

    els.push(html::new_text(&text[last_end..]));

    els
}
