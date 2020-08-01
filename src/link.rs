use kuchiki::NodeRef;
use std::collections::HashMap;

use crate::config;
use crate::html;
use crate::spec::Spec;

pub fn process_auto_links(doc: &mut Spec) {
    let auto_link_els = match doc
        .dom()
        .select("a:not([href]):not([data-link-type='biblio'])")
    {
        Ok(els) => els.map(|el| el.as_node().clone()).collect::<Vec<NodeRef>>(),
        _ => Vec::new(),
    };

    for auto_link_el in auto_link_els {
        html::insert_attr(&auto_link_el, "data-link-type", "dfn");

        let content = html::get_text_content(&auto_link_el);
        let name = config::generate_name(&content);
        html::insert_attr(
            &auto_link_el,
            "href",
            format!("https://drafts.csswg.org/css-flexbox-1/#{}", name),
        );
        html::insert_attr(&auto_link_el, "id", format!("ref-for-{}", name));

        let link_text = html::get_text_content(&auto_link_el);
        doc.link_texts.push(link_text);
    }
}

pub fn add_self_links(doc: &mut Spec) {
    let dfn_els = match doc.dom().select("dfn") {
        Ok(els) => els.map(|el| el.as_node().clone()).collect::<Vec<NodeRef>>(),
        _ => Vec::new(),
    };

    if let Ok(heading_els) = doc.dom().select("h2, h3, h4, h5, h6") {
        let mut found_first_numbered_section = false;

        for heading_el in heading_els {
            let heading_el = heading_el.as_node();

            found_first_numbered_section |= html::get_attr(heading_el, "data-level").is_some();

            if dfn_els.contains(heading_el) || !found_first_numbered_section {
                continue;
            }

            // Append self-link.
            if let Some(id) = html::get_attr(heading_el, "id") {
                let a_el = html::new_a(
                    btreemap! {
                        "class" => "self-link".to_owned(),
                        "href" => format!("#{}", id),
                    },
                    "",
                );
                heading_el.append(a_el);
            }
        }
    }

    add_dfn_panels(doc, &dfn_els);
}

fn add_dfn_panels(doc: &mut Spec, dfn_els: &[NodeRef]) {
    // id => <a> elements with this id
    let mut all_refs: HashMap<String, Vec<NodeRef>> = HashMap::new();

    if let Ok(a_els) = doc.dom().select("a") {
        for a_el in a_els {
            let a_el = a_el.as_node();

            let href = match html::get_attr(a_el, "href") {
                Some(href) => href,
                None => continue,
            };

            if !href.starts_with('#') {
                continue;
            }

            let refs = all_refs
                .entry(href[1..].to_owned())
                .or_insert_with(Vec::new);

            refs.push(a_el.to_owned());
        }
    }

    for dfn_el in dfn_els {
        let id = match html::get_attr(dfn_el, "id") {
            Some(id) => id,
            None => continue,
        };

        html::add_class(dfn_el, "css");
        html::insert_attr(dfn_el, "data-export", "");
        html::remove_attr(dfn_el, "property");

        // Insert a self-link.
        let a_el = html::new_a(
            btreemap! {
                "class" => "self-link".to_owned(),
                "href" => format!("#{}", id),
            },
            "",
        );
        dfn_el.append(a_el);
    }

    if !dfn_els.is_empty() {
        doc.extra_styles
            .insert("dfn-panel", include_str!("style/dfn-panel.css"));
        doc.extra_scripts
            .insert("dfn-panel", include_str!("script/dfn-panel.js"));
    }
}
