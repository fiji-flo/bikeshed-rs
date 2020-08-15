use regex::Regex;
use std::collections::HashMap;

use crate::config;
use crate::line::Line;
use crate::link::reference::Reference;
use crate::spec::Spec;
use crate::util;

pub fn transform_data_blocks(doc: &mut Spec, lines: &[Line]) -> Vec<Line> {
    lazy_static! {
        // regex for begin tag
        static ref BEGIN_TAG_REG: Regex = Regex::new(r"<pre [^>]*class=[^>]*anchors[^>]*>").unwrap();
        // regex for </pre> end tag
        static ref END_TAG_REG: Regex = Regex::new(r"</pre>\s*").unwrap();
    }

    let mut new_lines = Vec::new();
    let mut in_data_block = false;
    let mut data_block_lines = Vec::new();

    for line in lines {
        if BEGIN_TAG_REG.is_match(&line.text) && !in_data_block {
            // Meet begin tag.
            in_data_block = true;
        } else if END_TAG_REG.is_match(&line.text) && in_data_block {
            // Meet end tag.
            in_data_block = false;
            transform_anchors(doc, &data_block_lines);
            data_block_lines.clear();
        } else if in_data_block {
            // Handle line in data block.
            data_block_lines.push(line.clone());
        } else {
            // Handle line outside data block.
            new_lines.push(line.clone());
        }
    }

    new_lines
}

fn transform_anchors(doc: &mut Spec, lines: &[Line]) {
    let anchors = parse_info_tree(lines, doc.md.indent());
    process_anchors(doc, &anchors);
}

fn process_anchors(doc: &mut Spec, anchors: &[HashMap<String, Vec<String>>]) {
    // anchors:
    // [
    //     {
    //         "type" => ...,
    //         "text" => ...,
    //         ...
    //     }
    //     {
    //         "type" => ...,
    //         "text" => ...,
    //         ...
    //     }
    // ]

    for anchor in anchors {
        let link_type_vals = anchor.get("type").unwrap();
        let link_type = link_type_vals[0].to_owned();

        let link_text_vals = anchor.get("text").unwrap();
        let link_text = link_text_vals[0].to_owned();

        let url_prefix = if let Some(url_prefix_vals) = anchor.get("urlPrefix") {
            url_prefix_vals.join("")
        } else {
            "".to_owned()
        };

        let name = config::generate_name(&link_text);

        let reference = Reference {
            link_type,
            spec: None,
            url: format!("{}#{}", url_prefix, name),
            // TODO: fill link fors here.
            link_fors: Vec::new(),
        };

        doc.reference_manager
            .local_references
            .insert(link_text, reference);
    }
}

// Parse sets of info, which can be arranged into trees.
fn parse_info_tree(lines: &[Line], tab_size: u32) -> Vec<HashMap<String, Vec<String>>> {
    // Each info is a set of key-value pairs, semicolon-separated:
    // key1: val1; key2: val2; key3: val3

    // Intead of semicolon-separating, pieces can be nested with higher indentation:
    // key1: val1
    //     key2: val2
    //         key3: val3

    // Multiple fragments can be chained off of a single higher-level piece,
    // to avoid repetition:
    // key1: val1
    //     key2a: val2a
    //     key2b: val2b
    // ↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓↓
    // key1: val1; key2a: val2a
    // key1: val1; key2b: val2b

    lazy_static! {
        // regex for key-value pair
        static ref PAIR_REG: Regex = Regex::new(r"(?P<key>[^:]+):\s*(?P<val>.*)").unwrap();
    }

    // key => values
    let mut info_pairs: Vec<HashMap<String, Vec<String>>> = Vec::new();

    let mut extend_pairs = |level_pairs: &[HashMap<String, String>]| {
        let mut info_pair: HashMap<String, Vec<String>> = HashMap::new();

        for level_pair in level_pairs {
            for (key, val) in level_pair {
                info_pair
                    .entry(key.to_owned())
                    .or_default()
                    .push(val.to_owned());
            }
        }

        info_pairs.push(info_pair);
    };

    let mut last_indent_level = -1;
    // indent level => pair
    let mut level_pairs = Vec::new();

    for line in lines {
        let indent_level = util::indent::get_indent_level(&line.text, tab_size) as i32;

        if indent_level >= last_indent_level + 2 {
            die!(
                "Line jumps {} indent levels: {}.",
                indent_level - last_indent_level,
                line.text
            );
        }

        if indent_level <= last_indent_level {
            extend_pairs(&level_pairs[..=(last_indent_level as usize)]);
        }

        let text = util::indent::trim_indent(&line.text, indent_level as u32, tab_size);

        // TODO: Support grammar like `key:val; key:val; key:val`.
        let mut pair = HashMap::new();

        match PAIR_REG.captures(&text) {
            Some(caps) => {
                let key = caps["key"].trim().to_owned();
                let val = caps["val"].trim().to_owned();
                pair.insert(key, val);
            }
            None => die!("Line doesn't match the grammar: \"key: value\""),
        };

        if (indent_level as usize) < level_pairs.len() {
            level_pairs[indent_level as usize] = pair;
        } else {
            level_pairs.push(pair);
        }

        last_indent_level = indent_level;
    }

    extend_pairs(&level_pairs[..=(last_indent_level as usize)]);

    info_pairs
}
