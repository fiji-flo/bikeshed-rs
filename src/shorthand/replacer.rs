#![allow(clippy::trivial_regex)]

use kuchiki::NodeRef;
use regex::{Captures, Regex};

use crate::html::{self, Attr};

lazy_static! {
    // regex for var
    pub static ref VAR_REG: Regex = Regex::new(concat!(
        r"(?x)
        (?P<escape>\\)?
        \|
        (?P<inner_text>\w(?:[\w\s-]*\w)?)
        \|"
    ))
    .unwrap();
}

pub fn var_replacer(caps: &Captures) -> Vec<NodeRef> {
    if caps.name("escape").is_some() {
        return vec![html::new_text(&caps[0][1..])];
    }

    let var_el = html::new_element("var", None::<Attr>);
    var_el.append(html::new_text(&caps["inner_text"]));

    vec![var_el]
}

lazy_static! {
    // regex for inline link
    pub static ref INLINE_LINK_REG: Regex = Regex::new(concat!(
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
}

pub fn inline_link_replacer(caps: &Captures) -> Vec<NodeRef> {
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

lazy_static! {
    // regex for strong
    pub static ref STRONG_REG: Regex = Regex::new(
        r"(?x)
        ((?P<left>[^\\])|^)
        \*\*
        (?P<inner_text>[^\s][^*]*[^\s][^\\])
        \*\*"
    )
    .unwrap();
}

pub fn strong_replacer(caps: &Captures) -> Vec<NodeRef> {
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

lazy_static! {
    // regex for emphasis
    pub static ref EMPHASIS_REG: Regex = Regex::new(
        r"(?x)
        ((?P<left>[^\\*])|^)
        \*
        (?P<inner_text>[^\s][^*]*[^\s][^\\*])
        \*"
    )
    .unwrap();
}

pub fn emphasis_replacer(caps: &Captures) -> Vec<NodeRef> {
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

lazy_static! {
    // regex for escaped asterisk
    pub static ref ESCAPED_ASTERISK_REG: Regex = Regex::new(r"\\\*").unwrap();
}

pub fn escaped_asterisk_replacer(_: &Captures) -> Vec<NodeRef> {
    vec![html::new_text("*")]
}

lazy_static! {
    // regex for biblio link
    pub static ref BIBLIO_LINK_REG: Regex = Regex::new(
        r"(?x)
        (?P<escape>\\)?
        \[\[
        (?P<bang>!)?
        (?P<term>[\w.+-]+)
        (\s+(?P<status>current|snapshot))?
        (\|(?P<link_text>[^\]]+))?
        \]\]"
    )
    .unwrap();
}

pub fn biblio_link_replacer(caps: &Captures) -> Vec<NodeRef> {
    if caps.name("escape").is_some() {
        return vec![html::new_text(&caps[0][1..])];
    }

    let biblio_type = if caps.name("bang").is_some() {
        "normative"
    } else {
        "informative"
    };

    let term = &caps["term"];

    let link_text = match caps.name("link_text") {
        Some(m) => m.as_str().to_owned(),
        None => format!("[{}]", term),
    };

    let mut attrs = btreemap! {
        "data-lt" => term,
        "data-link-type" => "biblio",
        "data-biblio-type" => biblio_type,
    };

    if let Some(m) = caps.name("status") {
        attrs.insert("data-biblio-status", m.as_str());
    }

    vec![html::new_a(attrs, link_text)]
}
