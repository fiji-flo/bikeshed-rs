use kuchiki::NodeRef;

use crate::config::DFN_SELECTOR;
use crate::config::{self, DFN_TYPES, DFN_TYPE_TO_CLASS};
use crate::html;
use crate::spec::Spec;

pub fn process_dfns(doc: &mut Spec) {
    let dfn_els = html::select(doc.dom(), &DFN_SELECTOR).collect::<Vec<NodeRef>>();
    classify_dfns(&dfn_els);
    html::dedup_ids(doc.dom());
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

        // export or noexport
        if !html::has_attr(dfn_el, "data-export") && !html::has_attr(dfn_el, "data-noexport") {
            match html::get_closest_attr(dfn_el, &["data-export", "data-noexport"]) {
                Some((attr_name, _)) => html::insert_attr(dfn_el, attr_name, ""),
                None => {
                    if dfn_type == "dfn" {
                        html::insert_attr(dfn_el, "data-noexport", "");
                    } else {
                        html::insert_attr(dfn_el, "data-export", "");
                    }
                }
            }
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
