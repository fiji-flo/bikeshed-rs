use indexmap::map::IndexMap;
use kuchiki::traits::*;
use kuchiki::{NodeData, NodeRef};
use markup5ever::LocalName;
use std::char;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use crate::config::DFN_SELECTOR;
use crate::html::{self, Attr};
use crate::link::reference::Reference;
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

pub fn load_containers(doc: &mut Spec) {
    for container_el in html::select(doc.dom(), "[data-fill-with]") {
        doc.containers.insert(
            html::get_attr(&container_el, "data-fill-with").unwrap(),
            container_el,
        );
    }
}

fn get_container<'a>(doc: &'a Spec, tag: &str) -> Option<&'a NodeRef> {
    if !doc.md.boilerplate.get(tag) {
        return None;
    }

    doc.containers.get(tag)
}

fn get_container_or_head<'a>(doc: &'a Spec, tag: &str) -> Option<&'a NodeRef> {
    if !doc.md.boilerplate.get(tag) {
        return None;
    }

    doc.containers.get(tag).or_else(|| Some(doc.head()))
}

fn get_container_or_body<'a>(doc: &'a Spec, tag: &str) -> Option<&'a NodeRef> {
    if !doc.md.boilerplate.get(tag) {
        return None;
    }

    doc.containers.get(tag).or_else(|| Some(doc.body()))
}

pub fn add_header_footer(doc: &mut Spec) {
    let header = retrieve_boilerplate(doc, "header");
    let footer = retrieve_boilerplate(doc, "footer");
    doc.html = [header, doc.html.clone(), footer].join("\n");
}

pub fn add_styles(doc: &mut Spec) {
    // TODO: Insert <style> nodes to body and move them to head later.
    let container = match get_container_or_head(doc, "bs-styles") {
        Some(container) => container,
        None => return,
    };

    for (key, val) in doc.extra_styles.iter() {
        if doc.md.boilerplate.get(*key) {
            container.append(html::new_style(format!("/* style-{} */\n\n{}", key, val)));
        }
    }
}

pub fn add_scripts(doc: &mut Spec) {
    let container = match get_container_or_body(doc, "bs-scripts") {
        Some(container) => container,
        None => return,
    };

    for (key, val) in doc.extra_scripts.iter() {
        if doc.md.boilerplate.get(*key) {
            container.append(html::new_script(format!("/* script-{} */\n\n{}", key, val)));
        }
    }
}

pub fn add_canonical_url(doc: &mut Spec) {
    if let Some(ref canonical_url) = doc.md.canonical_url {
        doc.head().append(html::new_element(
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
    let dd_el = html::new_element(
        "dd",
        btreemap! {
            "class" => "editor p-author h-card vcard",
        },
    );

    if let Some(ref w3c_id) = editor.w3c_id {
        if let NodeData::Element(dd_el_data) = dd_el.data() {
            let attributes = &mut dd_el_data.attributes.borrow_mut();
            attributes.insert(LocalName::from("data-editor-id"), w3c_id.to_owned());
        }
    }

    if let Some(ref link) = editor.link {
        dd_el.append(html::new_a(
            btreemap! {
                "class" => "p-name fn u-url url",
                "href" => link,
            },
            &editor.name,
        ))
    } else if let Some(ref email) = editor.email {
        dd_el.append(html::new_a(
            btreemap! {
                "class" => "p-name fn u-email email".to_owned(),
                "href" => format!("mailto:{}", email),
            },
            &editor.name,
        ))
    } else {
        let span_el = html::new_element(
            "span",
            btreemap! {
                "class" => "p-name fn",
            },
        );
        span_el.append(html::new_text(&editor.name));
        dd_el.append(span_el);
    }

    if let Some(ref org) = editor.org {
        let el = if let Some(ref org_link) = editor.org_link {
            html::new_a(
                btreemap! {
                    "class" => "p-org org",
                    "href" => org_link,
                },
                org_link,
            )
        } else {
            let span_el = html::new_element(
                "span",
                btreemap! {
                    "class" => "p-org org",
                },
            );
            span_el.append(html::new_text(org.to_owned()));
            span_el
        };
        dd_el.append(html::new_text(" ("));
        dd_el.append(el);
        dd_el.append(html::new_text(")"));
    }

    if editor.link.is_some() {
        if let Some(ref email) = editor.email {
            dd_el.append(html::new_text(" "));
            dd_el.append(html::new_a(
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

pub fn fill_spec_metadata_section(doc: &mut Spec) {
    let container = match get_container(doc, "spec-metadata") {
        Some(container) => container,
        None => return,
    };

    fn key_to_dt_node(key: &str) -> NodeRef {
        let dt_el = match key {
            "Editor" => html::new_element(
                "dt",
                btreemap! {
                    "class" => "editor"
                },
            ),
            _ => html::new_element("dt", None::<Attr>),
        };
        dt_el.append(html::new_text(format!("{}:", key)));
        dt_el
    }

    fn wrap_in_dd_node(el: NodeRef) -> NodeRef {
        let dd_el = html::new_element("dd", None::<Attr>);
        dd_el.append(el);
        dd_el
    }

    let macros = &doc.macros;

    // <dt> and <dd> nodes that would be appended to <dl> node
    let mut md_list = Vec::new();

    // Insert version.
    if let Some(version) = macros.get("version") {
        md_list.push(key_to_dt_node("This version"));
        md_list.push(wrap_in_dd_node(html::new_a(
            btreemap! {
                "class" => "u-url",
                "href" => version,
            },
            version,
        )));
    }

    // Insert latest published version.
    if let Some(ref tr) = doc.md.tr {
        md_list.push(key_to_dt_node("Latest published version"));
        md_list.push(wrap_in_dd_node(html::new_a(
            btreemap! {
                "href" => tr
            },
            tr,
        )));
    }

    // Insert editors.
    if !doc.md.editors.is_empty() {
        md_list.push(key_to_dt_node("Editor"));
        md_list.extend(doc.md.editors.iter().map(editor_to_dd_node));
    }

    // Insert custom metadata.
    for (key, vals) in &doc.md.custom_md {
        md_list.push(key_to_dt_node(key));
        md_list.extend(vals.iter().map(|val| wrap_in_dd_node(html::new_text(val))));
    }

    let dl_el = html::new_element("dl", None::<Attr>);

    for item in md_list {
        dl_el.append(item);
    }

    container.append(dl_el);
}

pub fn fill_copyright_section(doc: &mut Spec) {
    let container = match get_container(doc, "copyright") {
        Some(container) => container,
        None => return,
    };

    let mut copyright = retrieve_boilerplate(doc, "copyright");
    copyright = doc.fix_text(&copyright);
    let copyright_dom = kuchiki::parse_html().one(copyright);

    if let Some(body) = html::select_first(&copyright_dom, "body") {
        for child in body.children() {
            container.append(child);
        }
    }
}

pub fn fill_abstract_section(doc: &mut Spec) {
    let container = match get_container(doc, "abstract") {
        Some(container) => container,
        None => return,
    };

    let mut abs = retrieve_boilerplate(doc, "abstract");
    abs = doc.fix_text(&abs);
    let abs_dom = kuchiki::parse_html().one(abs);

    if let Some(body) = html::select_first(&abs_dom, "body") {
        for child in body.children() {
            container.append(child);
        }
    }
}

pub fn add_index_section(doc: &mut Spec) {
    let mut dfn_els = html::select(doc.dom(), &DFN_SELECTOR);

    if dfn_els.next().is_none() {
        return;
    }

    let container = match get_container_or_body(doc, "index") {
        Some(container) => container.to_owned(),
        None => return,
    };

    let h2_el = html::new_element(
        "h2",
        btreemap! {
            "class" => "no-num no-ref",
            "id" => "index",
        },
    );
    h2_el.append(html::new_text("Index"));
    container.append(h2_el);

    add_local_terms(doc, &container);
    add_external_terms(doc, &container);
}

#[derive(Debug, Clone)]
struct IndexTerm {
    url: String,
    label: String,
    disambiguator: String,
}

fn index_items_to_node(index_items: IndexMap<String, IndexTerm>) -> NodeRef {
    let ul_el = html::new_element(
        "ul",
        btreemap! {
            "class" => "index",
        },
    );

    for (link_text, index_item) in index_items.sorted_by(|_, index_item1, _, index_item2| {
        if index_item1.disambiguator < index_item2.disambiguator {
            return Ordering::Less;
        }
        if index_item1.disambiguator == index_item2.disambiguator {
            return Ordering::Equal;
        }
        Ordering::Greater
    }) {
        let li_el = html::new_element("li", None::<Attr>);

        let a_el = html::new_a(
            btreemap! {
                "href" => &index_item.url,
            },
            link_text,
        );
        li_el.append(a_el);

        let span_el = html::new_element("span", None::<Attr>);
        span_el.append(html::new_text(format!(", in {}", index_item.label)));
        li_el.append(span_el);

        ul_el.append(li_el);
    }

    ul_el
}

fn add_local_terms(doc: &Spec, container: &NodeRef) {
    let h3_el = html::new_element(
        "h3",
        btreemap! {
            "class" => "no-num no-ref",
            "id" => "index-defined-here",
        },
    );
    h3_el.append(html::new_text("Terms defined by this specification"));
    container.append(h3_el);

    // link text => index item
    let mut index_items = IndexMap::new();

    for dfn_el in html::select(doc.dom(), &DFN_SELECTOR) {
        let link_text = html::get_text_content(&dfn_el);
        let id = html::get_attr(&dfn_el, "id").unwrap();
        let heading_level = "Unnumbered section";

        let dfn_type = html::get_attr(&dfn_el, "data-dfn-type").unwrap();
        let disambiguator = match dfn_type.as_str() {
            "dfn" => "definition of".to_owned(),
            _ => dfn_type,
        };

        index_items.insert(
            link_text,
            IndexTerm {
                url: format!("#{}", id),
                label: format!("§{}", heading_level),
                disambiguator,
            },
        );
    }

    container.append(index_items_to_node(index_items));
}

fn make_panel(reference: &Reference, name: &str, term_id: &str) -> NodeRef {
    let aside_el = html::new_element(
        "aside",
        btreemap! {
            "class" => "dfn-panel",
            "data-for" => term_id,
        },
    );

    let a_el = html::new_a(
        btreemap! {
            "href" => &reference.url
        },
        &reference.url,
    );
    aside_el.append(a_el);

    let b_el = html::new_element("b", None::<Attr>);
    b_el.append(html::new_text("Referenced in:"));
    aside_el.append(b_el);

    let ul_el = {
        let ul_el = html::new_element("ul", None::<Attr>);

        let li_el = html::new_element("li", None::<Attr>);
        let a_el = html::new_a(
            btreemap! {
                "href" => format!("#ref-for-{}", name),
            },
            "Unnamed section",
        );
        li_el.append(a_el);

        ul_el.append(li_el);

        ul_el
    };

    aside_el.append(ul_el);

    aside_el
}

fn add_external_terms(doc: &mut Spec, container: &NodeRef) {
    if doc.external_references_used.is_empty() {
        return;
    }

    let ul_el = html::new_element(
        "ul",
        btreemap! {
            "class" => "index",
        },
    );

    let mut at_least_one_panel = false;

    for (spec, references) in &doc.external_references_used {
        for (link_text, reference) in references {
            let name = reference.url.rsplitn(2, '#').next().unwrap();
            let term_id = format!("term-for-{}", name);

            let aside_el = make_panel(reference, name, &term_id);
            container.append(aside_el);

            let spec_li_el = {
                let spec_li_el = html::new_element("li", None::<Attr>);

                let a_el = html::new_a(
                    btreemap! {
                        "data-link-type" => "biblio"
                    },
                    format!("[{}]", spec),
                );
                spec_li_el.append(a_el);

                spec_li_el.append(html::new_text(" defines the following terms:"));

                let terms_ul_el = {
                    let ul_el = html::new_element("ul", None::<Attr>);

                    let li_el = html::new_element("li", None::<Attr>);

                    let span_el = html::new_element(
                        "span",
                        btreemap! {
                            "class" => "dfn-paneled",
                            "id" => &term_id,
                            "style" => "color:initial",
                        },
                    );
                    span_el.append(html::new_text(link_text));
                    li_el.append(span_el);

                    ul_el.append(li_el);

                    ul_el
                };
                spec_li_el.append(terms_ul_el);

                spec_li_el
            };

            ul_el.append(spec_li_el);

            at_least_one_panel = true;
        }
    }

    if at_least_one_panel {
        doc.extra_styles
            .insert("dfn-panel", include_str!("style/dfn-panel.css"));
        doc.extra_scripts
            .insert("dfn-panel", include_str!("script/dfn-panel.js"));
    }

    let h3_el = html::new_element(
        "h3",
        btreemap! {
            "class" => "no-num no-ref",
            "id" => "index-defined-elsewhere",
        },
    );
    h3_el.append(html::new_text("Terms defined by reference"));
    container.append(h3_el);

    container.append(ul_el);
}

pub fn add_references_section(doc: &mut Spec) {
    fn format_biblio_term(link_text: &str) -> String {
        if link_text
            .chars()
            .all(|ch| !char::is_alphabetic(ch) || char::is_lowercase(ch))
        {
            link_text.to_uppercase()
        } else {
            link_text.to_owned()
        }
    }

    if doc.normative_biblio_entries.is_empty() {
        return;
    }

    let container = match get_container_or_body(doc, "references") {
        Some(container) => container,
        None => return,
    };

    let h2_el = html::new_element(
        "h2",
        btreemap! {
            "class" => "no-num no-ref",
            "id" => "references",
        },
    );
    h2_el.append(html::new_text("References"));
    container.append(h2_el);

    let h3_el = html::new_element(
        "h3",
        btreemap! {
            "class" => "no-num no-ref",
            "id" => "normative",
        },
    );
    h3_el.append(html::new_text("Normative References"));
    container.append(h3_el);

    let dl_el = html::new_element("dl", None::<Attr>);

    for normative_biblio_entry in doc.normative_biblio_entries.values() {
        let id = format!("biblio-{}", normative_biblio_entry.link_text);

        let dt_el = html::new_element(
            "dt",
            btreemap! {
                "id" => id,
            },
        );
        dt_el.append(html::new_text(format!(
            "[{}]",
            format_biblio_term(&normative_biblio_entry.link_text)
        )));

        dl_el.append(dt_el);
        dl_el.append(normative_biblio_entry.to_node());
    }

    container.append(dl_el);
}

pub fn fill_toc_section(doc: &mut Spec) {
    let container = match get_container(doc, "table-of-contents") {
        Some(container) => container,
        None => return,
    };

    let h2_el = html::new_element(
        "h2",
        btreemap! {
            "class" => "no-num no-toc no-ref",
            "id" => "contents",
        },
    );
    h2_el.append(html::new_text("Table of Contents"));
    container.append(h2_el);

    // Each cell stores the reference to <ol> of a particular heading level.
    // Relation: <h[level]> => ol_cells[level - 2], where 2 <= level <= 6.
    let mut ol_cells: [Option<NodeRef>; 6] = Default::default();

    // Append a directory node (<ol> node) to table of contents, and then
    // store it to ol_cells[0].
    let dir_ol_el = html::new_element(
        "ol",
        btreemap! {
            "class" => "toc",
            "role"=> "directory",
        },
    );
    container.append(dir_ol_el.clone());
    ol_cells[0] = Some(dir_ol_el);

    let mut previous_level = 1;

    for heading_el in html::select(doc.dom(), "h2, h3, h4, h5, h6") {
        let heading_tag = html::get_tag(&heading_el).unwrap();
        let curr_level = heading_tag.chars().last().unwrap().to_digit(10).unwrap() as usize;

        if curr_level > previous_level + 1 {
            die!(
                "Heading level jumps more than one level, from h{} to h{}",
                previous_level,
                curr_level
            )
        }

        let curr_ol_el = if let Some(ref curr_ol_el) = ol_cells[curr_level - 2] {
            curr_ol_el
        } else {
            die!(
                "Saw an <h{}> without seeing an <h{}> first. Please order your headings properly.",
                curr_level,
                curr_level - 1
            )
        };

        if html::has_class(&heading_el, "no-toc") {
            ol_cells[curr_level - 1] = None;
        } else {
            // Add a <li> node to current <ol> node.
            let a_el = {
                let a_el = html::new_a(
                    btreemap! {
                        "href" => format!("#{}", html::get_attr(&heading_el, "id").unwrap())
                    },
                    "",
                );

                let span_el = html::new_element(
                    "span",
                    btreemap! {
                        "class"=>"secno"
                    },
                );
                span_el.append(html::new_text(
                    html::get_attr(&heading_el, "data-level").unwrap_or_default(),
                ));
                a_el.append(span_el);

                a_el.append(html::new_text(" "));

                if let Some(content_el) = html::select_first(&heading_el, ".content") {
                    a_el.append(html::deep_clone(&content_el));
                }

                a_el
            };

            let li_el = html::new_element("li", None::<Attr>);
            li_el.append(a_el);

            let inner_ol_el = html::new_element(
                "ol",
                btreemap! {
                    "class" => "toc",
                },
            );
            li_el.append(inner_ol_el.clone());

            curr_ol_el.append(li_el);

            ol_cells[curr_level - 1] = Some(inner_ol_el);
        }

        previous_level = curr_level;
    }

    // Remove empty <ol> nodes.
    loop {
        let ol_els = html::select(container, "ol:empty").collect::<Vec<NodeRef>>();

        if ol_els.is_empty() {
            break;
        }

        for ol_el in ol_els {
            ol_el.detach();
        }
    }
}
