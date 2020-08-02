use kuchiki::NodeRef;

use crate::config::{self, DFN_TYPES, DFN_TYPE_TO_CLASS};
use crate::html;
use crate::spec::Spec;

pub fn process_dfns(doc: &mut Spec) {
    let dfn_els = html::select(doc.dom(), "dfn").collect::<Vec<NodeRef>>();
    classify_dfns(&dfn_els);
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

        if !DFN_TYPES.contains(dfn_type.as_str()) {
            die!("Unknown dfn type: {}.", dfn_type);
        }

        if !html::has_attr(dfn_el, "data-dfn-type") {
            html::insert_attr(dfn_el, "data-dfn-type", &dfn_type);
        }

        // Fill in id if necessary.
        if dfn_type != "dfn" && !html::has_attr(dfn_el, "id") {
            let dfn_class = DFN_TYPE_TO_CLASS.get(dfn_type.as_str()).unwrap();
            let dfn_text = html::get_text_content(dfn_el);
            let name = config::generate_name(&dfn_text);
            html::insert_attr(dfn_el, "id", format!("{}-{}", dfn_class, name));
        }
    }
}
