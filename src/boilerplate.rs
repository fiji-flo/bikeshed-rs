use std::fs;
use std::path::Path;

use crate::html;
use crate::spec::Spec;

// Retrieve boilerplate file with doc (metadata).
fn retrieve_boilerplate(doc: &mut Spec, name: &str) -> String {
    retrieve_boilerplate_with_info(name, doc.md.group.clone(), doc.md.raw_status.clone())
}

// Retrieve boilerplate file with group and status.
fn retrieve_boilerplate_with_info(
    name: &str,
    group: Option<String>,
    status: Option<String>,
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
        if let Some(group) = group.clone() {
            paths_to_try.push(Path::new("boilerplate").join(group).join(status_filename));
        }
    }

    let generic_filename = format!("{}.include", name);

    if let Some(group) = group {
        // generic file with group
        paths_to_try.push(
            Path::new("boilerplate")
                .join(group)
                .join(generic_filename.clone()),
        );
    }

    if let Some(ref status_filename) = status_filename {
        // status file without group
        paths_to_try.push(Path::new("boilerplate").join(status_filename));
    }

    // generic file without group
    paths_to_try.push(Path::new("boilerplate").join(generic_filename));

    for path in paths_to_try {
        if path.is_file() {
            match fs::read_to_string(path) {
                Ok(data) => {
                    return data;
                }
                Err(_) => die!("Can't read data from the include file for {}.", name),
            }
        }
    }

    die!("Can't find an appropriate include file for {}.", name);
}

pub fn add_header_footer(doc: &mut Spec) {
    let header = if doc.md.boilerplate.get("header") {
        retrieve_boilerplate(doc, "header")
    } else {
        String::new()
    };

    let footer = if doc.md.boilerplate.get("footer") {
        retrieve_boilerplate(doc, "footer")
    } else {
        String::new()
    };

    doc.html = [header, doc.html.clone(), footer].join("\n");
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
