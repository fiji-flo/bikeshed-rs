use kuchiki::NodeRef;
use std::cmp;

use crate::config;
use crate::html;
use crate::spec::Spec;

pub fn process_headings(doc: &mut Spec) {
    // TODO:
    // 1. [all] or [doc-only]?
    // 2. [settled] or not?

    let heading_els = html::select(doc.dom(), "h2, h3, h4, h5, h6").collect::<Vec<NodeRef>>();

    for heading_el in &heading_els {
        html::add_class(heading_el, "heading");
        html::add_class(heading_el, "settled");
        wrap_heading_contents(heading_el);
    }

    add_level(&heading_els);
    add_default_id(&heading_els);
    add_secno(&heading_els);
}

// Wrap the content of heading into a <span class="content"> element.
fn wrap_heading_contents(heading_el: &NodeRef) {
    let content_el = html::new_element(
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

        for heading_level in heading_levels.iter_mut().take(5).skip(level - 1) {
            *heading_level = 0;
        }
    };

    fn levels_to_string(heading_levels: &[u32]) -> String {
        heading_levels
            .iter()
            .filter(|&l| l > &0)
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
            .join(".")
    };

    let mut heading_levels: Vec<u32> = vec![0, 0, 0, 0, 0];

    let mut skip_level = u32::max_value();

    for heading_el in heading_els {
        let heading_tag = html::get_tag(heading_el).unwrap();
        let level = heading_tag.chars().last().unwrap().to_digit(10).unwrap();

        if html::has_class(heading_el, "no-num") {
            // Ignore headings with "no-num" class.
            skip_level = cmp::min(skip_level, level);
            continue;
        }

        // Skip headings that are in the same section with the ignored headings.
        if level > skip_level {
            continue;
        }

        skip_level = u32::max_value();

        increment_level(&mut heading_levels, level as usize);
        html::insert_attr(heading_el, "data-level", levels_to_string(&heading_levels));
    }
}

// Insert default "id" attribute into each heading element if necessary.
fn add_default_id(heading_els: &[NodeRef]) {
    for heading_el in heading_els {
        if html::has_attr(heading_el, "id") {
            continue;
        }

        // Generate id from content.
        if let Some(content_el) = html::select_first(heading_el, ".content") {
            let content = html::get_text_content(&content_el);
            html::insert_attr(heading_el, "id", config::generate_name(&content))
        }
    }
}

// Prepend a <span class="secno"> element to each heading element.
fn add_secno(heading_els: &[NodeRef]) {
    for heading_el in heading_els {
        if let Some(data_level) = html::get_attr(heading_el, "data-level") {
            // Prepend secno.
            let span_el = html::new_element(
                "span",
                btreemap! {
                    "class" => "secno",
                },
            );
            span_el.append(html::new_text(format!("{}. ", data_level)));
            heading_el.prepend(span_el);
        }
    }
}
