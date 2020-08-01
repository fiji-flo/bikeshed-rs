use kuchiki::NodeRef;

use crate::config;
use crate::html;
use crate::spec::Spec;

pub fn process_dfns(doc: &mut Spec) {
    let dfn_els = match doc.dom().select("dfn") {
        Ok(els) => els.map(|el| el.as_node().clone()).collect::<Vec<NodeRef>>(),
        _ => Vec::new(),
    };

    classify_dfns(&dfn_els);
}

fn classify_dfns(dfn_els: &[NodeRef]) {
    for dfn_el in dfn_els {
        let dfn_type = "property";

        if !html::has_attr(dfn_el, "data-dfn-type") {
            html::insert_attr(dfn_el, "data-dfn-type", dfn_type);
        }

        // Fill in id if necessary.
        if !html::has_attr(dfn_el, "id") {
            let content = html::get_text_content(dfn_el);
            let id = format!("propdef-{}", config::generate_name(&content));
            html::insert_attr(dfn_el, "id", id);
        }
    }
}
