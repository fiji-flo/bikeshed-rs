#![allow(clippy::trivial_regex)]

use kuchiki::NodeRef;
use regex::{Captures, Regex};

use crate::html::{self, Attr};
use crate::spec::Spec;
use crate::util::boolset::BoolSet;

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

// Life would be easier if the look-around syntax was supported by the regex library.
lazy_static! {
    // regex for var
    static ref VAR_REG: Regex = Regex::new(concat!(
        r"(?x)
        (?P<escape>\\)?
        \|
        (?P<inner_text>\w(?:[\w\s-]*\w)?)
        \|"
    ))
    .unwrap();
    // regex for inline link
    static ref INLINE_LINK_REG: Regex = Regex::new(concat!(
        r"(?x)
            (\\)?
            \[
            (?P<text>[^\]]*)
            \]
            \(\s*
            (?P<href>[^\s)]+)",
        r#"\s*("
            (?P<title>[^"]*)
            ")?\s*"#,
        r"\)"
    ))
    .unwrap();
    // regex for strong
    static ref STRONG_REG: Regex = Regex::new(
        r"(?x)
        ((?P<left>[^\\])|^)
        \*\*
        (?P<inner_text>[^\s][^*]*[^\s][^\\])
        \*\*"
    )
    .unwrap();
    // regex for emphasis
    static ref EMPHASIS_REG: Regex = Regex::new(
        r"(?x)
        ((?P<left>[^\\*])|^)
        \*
        (?P<inner_text>[^\s][^*]*[^\s][^\\*])
        \*"
    )
    .unwrap();
    // regex for escaped asterisk
    static ref ESCAPED_ASTERISK_REG: Regex = Regex::new(r"\\\*").unwrap();
}

fn var_replacer(caps: &Captures) -> Vec<NodeRef> {
    if caps.name("escape").is_some() {
        return vec![html::new_text(&caps[0][1..])];
    }

    let var_el = html::new_element("var", None::<Attr>);
    var_el.append(html::new_text(&caps["inner_text"]));

    vec![var_el]
}

fn inline_link_replacer(caps: &Captures) -> Vec<NodeRef> {
    let href = &caps["href"];

    let attrs = match caps.name("title") {
        Some(title) => btreemap! {
            "href" => href,
            "title" => title.as_str(),
        },
        None => btreemap! {
            "href" => href
        },
    };

    let text = &caps["text"];
    let a_el = html::new_a(attrs, text);

    vec![a_el]
}

fn strong_replacer(caps: &Captures) -> Vec<NodeRef> {
    let mut els = Vec::new();

    if let Some(left_left) = caps.name("left") {
        els.push(html::new_text(left_left.as_str()));
    }

    let strong_el = {
        let strong_el = html::new_element("strong", None::<Attr>);
        let inner_text = &caps["inner_text"];
        strong_el.append(html::new_text(inner_text));
        strong_el
    };
    els.push(strong_el);

    els
}

fn emphasis_replacer(caps: &Captures) -> Vec<NodeRef> {
    let mut els = Vec::new();

    if let Some(left_left) = caps.name("left") {
        els.push(html::new_text(left_left.as_str()));
    }

    let em_el = {
        let em_el = html::new_element("em", None::<Attr>);
        let inner_text = &caps["inner_text"];
        em_el.append(html::new_text(inner_text));
        em_el
    };
    els.push(em_el);

    els
}

fn escaped_asterisk_replacer(_: &Captures) -> Vec<NodeRef> {
    vec![html::new_text("*")]
}

fn transform_text_node(text_el: &NodeRef, markup_shorthands: &BoolSet<String>) -> Vec<NodeRef> {
    let mut text_els = vec![text_el.clone()];

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
