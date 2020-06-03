use kuchiki::traits::*;
use kuchiki::NodeRef;
use std::fs;
use std::path::Path;

use crate::html;
use crate::metadata::parse::Editor;
use crate::spec::Spec;

// Retrieve boilerplate file with doc (metadata).
fn retrieve_boilerplate(doc: &Spec, name: &str) -> String {
    if doc.md.boilerplate.get(name) {
        retrieve_boilerplate_with_info(name, doc.md.group.as_deref(), doc.md.raw_status.as_deref())
    } else {
        String::new()
    }
}

// Retrieve boilerplate file with group and status.
pub fn retrieve_boilerplate_with_info(
    name: &str,
    group: Option<&str>,
    status: Option<&str>,
) -> String {
    // File Priorities:
    // 1. [status file with group]
    // 2. [generic file with group]
    // 3. [status file without group]
    // 4. [generic file without group]

    let mut paths_to_try = Vec::new();

    let status_filename = if let Some(status) = status {
        Some(format!("{}-{}.include", name, status))
    } else {
        None
    };

    if let Some(ref status_filename) = status_filename {
        // status file with group
        if let Some(group) = group {
            paths_to_try.push(Path::new("boilerplate").join(group).join(status_filename));
        }
    }

    let generic_filename = format!("{}.include", name);

    if let Some(group) = group {
        // generic file with group
        paths_to_try.push(Path::new("boilerplate").join(group).join(&generic_filename));
    }

    if let Some(ref status_filename) = status_filename {
        // status file without group
        paths_to_try.push(Path::new("boilerplate").join(status_filename));
    }

    // generic file without group
    paths_to_try.push(Path::new("boilerplate").join(&generic_filename));

    for path in paths_to_try {
        if let Ok(data) = fs::read_to_string(path) {
            return data;
        }
    }

    die!("Can't find an appropriate include file for {}.", name);
}

pub fn add_header_footer(doc: &mut Spec) {
    let header = retrieve_boilerplate(doc, "header");
    let footer = retrieve_boilerplate(doc, "footer");
    doc.html = [header, doc.html.clone(), footer].join("\n");
}

pub fn add_bikeshed_boilerplate(doc: &mut Spec) {
    // TODO: insert <style> nodes to body and move them to head later
    for (key, val) in doc.extra_styles.iter() {
        doc.head.as_ref().unwrap().append(html::node::new_style(
            format!("/* style-{} */\n{}", key, val).as_str(),
        ));
    }
}

pub fn add_canonical_url(doc: &mut Spec) {
    if let Some(canonical_url) = &doc.md.canonical_url {
        doc.head.as_ref().unwrap().append(html::node::new_element(
            "link",
            btreemap! {
                "rel" => "canonical".to_string(),
                "href" => canonical_url.to_string(),
            },
        ))
    }
}

fn editor_to_dd_node(editor: &Editor) -> NodeRef {
    let dd_el = html::node::new_element(
        "dd",
        btreemap! {
            "class" => "editor p-author h-card vcard".to_owned(),
        },
    );
    if editor.email.is_none() {
        let span_el = html::node::new_element(
            "span",
            btreemap! {
                "class" => "p-name fn".to_owned(),
            },
        );
        span_el.append(html::node::new_text(&editor.name));
        dd_el.append(span_el);
    }
    dd_el
}

pub fn add_spec_metadata_section(doc: &mut Spec) {
    let macros = &doc.macros;

    if let Some(ref mut dom) = doc.dom {
        if let Ok(ref container) = dom.select_first("div[data-fill-with=spec-metadata]") {
            let dl_el = html::node::new_element("dl", None);

            // insert version
            if let Some(version) = macros.get("version") {
                let dt_el = html::node::new_element("dt", None);
                dt_el.append(html::node::new_text("This version:"));
                dl_el.append(dt_el);

                let a_el = html::node::new_element(
                    "a",
                    btreemap! {
                        "class" => "u-url".to_owned(),
                        "href" => version.to_owned(),
                    },
                );
                a_el.append(html::node::new_text(version));
                let dd_el = html::node::new_element("dd", None);
                dd_el.append(a_el);
                dl_el.append(dd_el);
            }

            // insert editors
            if !doc.md.editors.is_empty() {
                let dt_el = html::node::new_element(
                    "dt",
                    btreemap! {
                        "class" => "editor".to_owned()
                    },
                );
                dt_el.append(html::node::new_text("Editor:"));
                dl_el.append(dt_el);

                for dd_el in doc.md.editors.iter().map(editor_to_dd_node) {
                    dl_el.append(dd_el);
                }
            }

            container.as_node().append(dl_el);
        }
    }
}

pub fn add_copyright_section(doc: &mut Spec) {
    if let Some(ref mut dom) = doc.dom {
        if let Ok(ref container) = dom.select_first("p[data-fill-with=copyright]") {
            let copyright = retrieve_boilerplate(doc, "copyright");
            let copyright_dom = kuchiki::parse_html().one(copyright);

            if let Ok(body) = copyright_dom.select_first("body") {
                for child in body.as_node().children() {
                    container.as_node().append(child);
                }
            }
        }
    }
}

pub fn add_abstract_section(doc: &mut Spec) {
    if let Some(ref mut dom) = doc.dom {
        if let Ok(ref container) = dom.select_first("div[data-fill-with=abstract]") {
            let mut abs = retrieve_boilerplate(doc, "abstract");
            abs = html::helper::replace_macros(&abs, &doc.macros);
            let abs_dom = kuchiki::parse_html().one(abs);

            if let Ok(body) = abs_dom.select_first("body") {
                for child in body.as_node().children() {
                    container.as_node().append(child);
                }
            }
        }
    }
}

pub fn add_toc_section(doc: &mut Spec) {
    if let Some(ref mut dom) = doc.dom {
        if let Ok(ref container) = dom.select_first("nav[data-fill-with=table-of-contents]") {
            let h2_el = html::node::new_element(
                "h2",
                btreemap! {
                    "class" => "no-num no-toc no-ref".to_owned(),
                    "id" => "contents".to_owned(),
                },
            );
            h2_el.append(html::node::new_text("Table of Contents"));
            container.as_node().append(h2_el);
        }
    }
}
