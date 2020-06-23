use kuchiki::NodeRef;
use std::cmp;

use crate::html;
use crate::spec::Spec;

pub fn process_headings(doc: &mut Spec) {
    // TODO:
    // 1. [all] or [doc-only]?
    // 2. [settled] or not?

    if let Ok(els) = doc.dom().select("h2, h3, h4, h5, h6") {
        let heading_els = els.map(|el| el.as_node().clone()).collect::<Vec<NodeRef>>();

        for heading_el in &heading_els {
            html::node::add_class(heading_el, "heading");
            html::node::add_class(heading_el, "settled");
            wrap_heading_contents(heading_el);
        }

        add_level(&heading_els);
        add_secno_and_self_link(&heading_els);
    }
}

// Wrap the content of heading into a <span class="content"> element.
fn wrap_heading_contents(heading_el: &NodeRef) {
    let content_el = html::node::new_element(
        "span",
        btreemap! {
            "class" => "content",
        },
    );

    for child in heading_el.children() {
        content_el.append(child);
    }

    heading_el.append(content_el);
}

// Insert "data-level" attribute into each heading element.
fn add_level(heading_els: &[NodeRef]) {
    fn increment_level(heading_levels: &mut [u32], level: usize) {
        heading_levels[level - 2] += 1;

        for i in (level - 1)..5 {
            heading_levels[i] = 0;
        }
    };

    fn levels_to_string(heading_levels: &[u32]) -> String {
        heading_levels
            .iter()
            .filter(|l| l.to_owned() > &0)
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
            .join(".")
    };

    let mut heading_levels: Vec<u32> = vec![0, 0, 0, 0, 0];

    let mut skip_level = u32::max_value();

    for heading_el in heading_els {
        let heading_tag = html::node::get_tag(heading_el).unwrap();
        // heading number
        let level = heading_tag.chars().last().unwrap().to_digit(10).unwrap();

        if html::node::has_class(heading_el, "no-num") {
            // ignore headings with "no-num" class
            skip_level = cmp::min(skip_level, level);
            continue;
        }

        // skip headings that are in the same section with the ignored headings
        if level > skip_level {
            continue;
        }

        skip_level = u32::max_value();

        increment_level(&mut heading_levels, level as usize);
        html::node::insert_attr(heading_el, "data-level", levels_to_string(&heading_levels));
    }
}

// Prepend a <span class="secno"> element to each heading element,
// and append an <a class="self-link"> element to each heading element.
fn add_secno_and_self_link(heading_els: &[NodeRef]) {
    for heading_el in heading_els {
        if let Some(data_level) = html::node::get_attr(heading_el, "data-level") {
            // prepend secno
            let span_el = html::node::new_element(
                "span",
                btreemap! {
                    "class" => "secno",
                },
            );
            span_el.append(html::node::new_text(format!("{}. ", data_level)));
            heading_el.prepend(span_el);

            // append self-link
            if let Some(id) = html::node::get_attr(heading_el, "id") {
                let a_el = html::node::new_a(
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
}
