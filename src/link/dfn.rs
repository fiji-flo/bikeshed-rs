use kuchiki::NodeRef;
use std::collections::HashMap;

use crate::config::{self, DFN_TYPES, DFN_TYPE_TO_CLASS};
use crate::html;
use crate::spec::Spec;

pub fn process_dfns(doc: &mut Spec) {
    let dfn_els = html::select(doc.dom(), "dfn").collect::<Vec<NodeRef>>();
    classify_dfns(&dfn_els);
    dedup_ids(doc.dom());
    doc.reference_manager.add_local_dfns(&dfn_els);
}

fn determine_dfn_type(dfn_el: &NodeRef) -> String {
    match html::get_attr(dfn_el, "data-dfn-type") {
        Some(dfn_type) => dfn_type,
        None => "dfn".to_owned(),
    }
}

fn classify_dfns(dfn_els: &[NodeRef]) {
    for dfn_el in dfn_els {
        let dfn_type = determine_dfn_type(dfn_el);

        assert!(
            DFN_TYPES.contains(dfn_type.as_str()),
            "Unknown dfn type: {}.",
            dfn_type
        );

        if !html::has_attr(dfn_el, "data-dfn-type") {
            html::insert_attr(dfn_el, "data-dfn-type", &dfn_type);
        }

        // Fill in id if necessary.
        if !html::has_attr(dfn_el, "id") {
            let dfn_text = html::get_text_content(dfn_el);
            let name = config::generate_name(&dfn_text);

            let id = if let Some(dfn_class) = DFN_TYPE_TO_CLASS.get(dfn_type.as_str()) {
                format!("{}-{}", dfn_class, name)
            } else {
                name
            };

            html::insert_attr(dfn_el, "id", id);
        }
    }
}

fn dedup_ids(root: &NodeRef) {
    fn to_circled_digits(num: usize) -> String {
        let digits = vec!["⓪", "①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨"];

        // TODO: Work on num directly.
        num.to_string()
            .chars()
            .map(|ch| digits[ch.to_digit(10).unwrap() as usize])
            .collect::<Vec<&str>>()
            .join("")
    }

    // id => nodes
    let mut ids: HashMap<String, Vec<NodeRef>> = HashMap::new();

    for el in html::select(root, "[id]") {
        let id = html::get_attr(&el, "id").unwrap();
        ids.entry(id).or_default().push(el);
    }

    for (id, els) in ids {
        if els.len() <= 1 {
            continue;
        }

        for (i, el) in els.iter().enumerate().skip(1) {
            to_circled_digits(i);
            let new_id = format!("{}{}", id, to_circled_digits(i));
            html::insert_attr(el, "id", new_id);
        }
    }
}
