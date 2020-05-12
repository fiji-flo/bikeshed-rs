use std::fs;

use crate::html;
use crate::spec::Spec;

pub fn add_header_footer(data: &mut String) {
    // TODO: handle group and status
    let header_path = "boilerplate/header.include";
    let footer_path = "boilerplate/footer.include";
    let header = fs::read_to_string(header_path).expect("Fail to open header file");
    let footer = fs::read_to_string(footer_path).expect("Fail to open footer file");
    *data = [header, data.clone(), footer].join("\n");
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
