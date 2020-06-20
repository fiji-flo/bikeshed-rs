use kuchiki::traits::*;
use kuchiki::{NodeData, NodeRef};
use markup5ever::LocalName;
use std::fs;
use std::path::Path;

use crate::html::{self, node::Attr};
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
        doc.head().append(html::node::new_style(format!(
            "/* style-{} */\n\n{}",
            key, val
        )));
    }
}

pub fn add_canonical_url(doc: &mut Spec) {
    if let Some(ref canonical_url) = doc.md.canonical_url {
        doc.head().append(html::node::new_element(
            "link",
            btreemap! {
                "rel" => "canonical",
                "href" => canonical_url,
            },
        ))
    }
}

// Convert an editor to a <dd> node.
fn editor_to_dd_node(editor: &Editor) -> NodeRef {
    let dd_el = html::node::new_element(
        "dd",
        btreemap! {
            "class" => "editor p-author h-card vcard",
        },
    );

    if let Some(ref w3c_id) = editor.w3c_id {
        if let NodeData::Element(dd_el_data) = dd_el.data() {
            let ref mut attributes = dd_el_data.attributes.borrow_mut();
            attributes.insert(LocalName::from("data-editor-id"), w3c_id.to_owned());
        }
    }

    if let Some(ref link) = editor.link {
        dd_el.append(html::node::new_a(
            btreemap! {
                "class" => "p-name fn u-url url",
                "href" => link,
            },
            &editor.name,
        ))
    } else if let Some(ref email) = editor.email {
        dd_el.append(html::node::new_a(
            btreemap! {
                "class" => "p-name fn u-email email".to_owned(),
                "href" => format!("mailto:{}", email),
            },
            &editor.name,
        ))
    } else {
        let span_el = html::node::new_element(
            "span",
            btreemap! {
                "class" => "p-name fn",
            },
        );
        span_el.append(html::node::new_text(&editor.name));
        dd_el.append(span_el);
    }

    if let Some(ref org) = editor.org {
        let el = if let Some(ref org_link) = editor.org_link {
            html::node::new_a(
                btreemap! {
                    "class" => "p-org org",
                    "href" => org_link,
                },
                org_link,
            )
        } else {
            let span_el = html::node::new_element(
                "span",
                btreemap! {
                    "class" => "p-org org",
                },
            );
            span_el.append(html::node::new_text(org.to_owned()));
            span_el
        };
        dd_el.append(html::node::new_text(" ("));
        dd_el.append(el);
        dd_el.append(html::node::new_text(")"));
    }

    if editor.link.is_some() {
        if let Some(ref email) = editor.email {
            dd_el.append(html::node::new_text(" "));
            dd_el.append(html::node::new_a(
                btreemap! {
                    "class" => "u-email email".to_owned(),
                    "href" => format!("mailto:{}", email),
                },
                email,
            ));
        }
    }

    dd_el
}

pub fn add_spec_metadata_section(doc: &mut Spec) {
    let container = match doc.dom().select_first("div[data-fill-with=spec-metadata]") {
        Ok(container) => container,
        Err(_) => return,
    };

    fn key_to_dt_node(key: &str) -> NodeRef {
        let dt_el = match key {
            "Editor" => html::node::new_element(
                "dt",
                btreemap! {
                    "class" => "editor"
                },
            ),
            _ => html::node::new_element("dt", None::<Attr>),
        };
        dt_el.append(html::node::new_text(format!("{}:", key)));
        dt_el
    }

    fn wrap_in_dd_node(el: NodeRef) -> NodeRef {
        let dd_el = html::node::new_element("dd", None::<Attr>);
        dd_el.append(el);
        dd_el
    }

    let macros = &doc.macros;

    // <dt> and <dd> nodes that would be appended to <dl> node
    let mut md_list = Vec::new();

    // insert version
    if let Some(version) = macros.get("version") {
        md_list.extend(vec![
            key_to_dt_node("This version"),
            wrap_in_dd_node(html::node::new_a(
                btreemap! {
                    "class" => "u-url",
                    "href" => version,
                },
                version,
            )),
        ]);
    }

    // insert latest published version
    if let Some(ref tr) = doc.md.tr {
        md_list.extend(vec![
            key_to_dt_node("Latest published version"),
            wrap_in_dd_node(html::node::new_a(
                btreemap! {
                    "href" => tr
                },
                tr,
            )),
        ]);
    }

    // insert editors
    if !doc.md.editors.is_empty() {
        md_list.push(key_to_dt_node("Editor"));
        md_list.extend(doc.md.editors.iter().map(editor_to_dd_node));
    }

    // insert custom metadata
    for (key, vals) in &doc.md.custom_md {
        md_list.push(key_to_dt_node(key));
        md_list.extend(
            vals.iter()
                .map(|val| wrap_in_dd_node(html::node::new_text(val))),
        );
    }

    let dl_el = html::node::new_element("dl", None::<Attr>);

    for item in md_list {
        dl_el.append(item)
    }

    container.as_node().append(dl_el);
}

pub fn add_copyright_section(doc: &mut Spec) {
    let container = match doc.dom().select_first("p[data-fill-with=copyright]") {
        Ok(container) => container,
        Err(_) => return,
    };

    let copyright = retrieve_boilerplate(doc, "copyright");
    let copyright_dom = kuchiki::parse_html().one(copyright);

    if let Ok(body) = copyright_dom.select_first("body") {
        for child in body.as_node().children() {
            container.as_node().append(child);
        }
    }
}

pub fn add_abstract_section(doc: &mut Spec) {
    let container = match doc.dom().select_first("div[data-fill-with=abstract]") {
        Ok(container) => container,
        Err(_) => return,
    };

    let mut abs = retrieve_boilerplate(doc, "abstract");
    abs = html::helper::replace_macros(&abs, &doc.macros);
    let abs_dom = kuchiki::parse_html().one(abs);

    if let Ok(body) = abs_dom.select_first("body") {
        for child in body.as_node().children() {
            container.as_node().append(child);
        }
    }
}

pub fn add_toc_section(doc: &mut Spec) {
    let container = match doc
        .dom()
        .select_first("nav[data-fill-with=table-of-contents]")
    {
        Ok(container) => container,
        Err(_) => return,
    };

    let h2_el = html::node::new_element(
        "h2",
        btreemap! {
            "class" => "no-num no-toc no-ref",
            "id" => "contents",
        },
    );
    h2_el.append(html::node::new_text("Table of Contents"));
    container.as_node().append(h2_el);
}
