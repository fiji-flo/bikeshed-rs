use kuchiki::{Attribute, ExpandedName, NodeData, NodeRef};
use markup5ever::{LocalName, QualName};
use std::collections::{HashMap, HashSet};

// (attr name, attr value)
pub type Attr<'a> = (&'a str, String);

pub fn new_element<'a, I, V>(name: &str, attributes: I) -> NodeRef
where
    I: IntoIterator<Item = (&'a str, V)>,
    V: Into<String>,
{
    NodeRef::new_element(
        QualName::new(None, ns!(html), LocalName::from(name)),
        attributes.into_iter().map(|(key, val)| {
            let expanded_name = ExpandedName::new(ns!(), LocalName::from(key));
            let attribute = Attribute {
                prefix: None,
                value: val.into(),
            };
            (expanded_name, attribute)
        }),
    )
}

pub fn new_text<T: Into<String>>(text: T) -> NodeRef {
    kuchiki::NodeRef::new_text(text)
}

pub fn new_style<T: Into<String>>(text: T) -> NodeRef {
    let el = new_element("style", None::<Attr>);
    el.append(NodeRef::new_text(text));
    el
}

pub fn new_script<T: Into<String>>(text: T) -> NodeRef {
    let el = new_element("script", None::<Attr>);
    el.append(NodeRef::new_text(text));
    el
}

pub fn new_a<'a, I, V, T>(attributes: I, text: T) -> NodeRef
where
    I: IntoIterator<Item = (&'a str, V)>,
    V: Into<String>,
    T: Into<String>,
{
    let el = new_element("a", attributes);
    el.append(NodeRef::new_text(text));
    el
}

pub fn is_text_node(el: &NodeRef) -> bool {
    el.as_text().is_some()
}

pub fn unwrap_text_node(el: &NodeRef) -> String {
    el.as_text().unwrap().borrow().clone()
}

pub fn replace_node(old_el: &NodeRef, new_el: &NodeRef) {
    old_el.insert_before(new_el.clone());
    old_el.detach();
}

pub fn get_tag(el: &NodeRef) -> Option<String> {
    match el.data() {
        NodeData::Element(data) => Some(data.name.local.to_string()),
        _ => None,
    }
}

pub fn has_attr(el: &NodeRef, attr_name: &str) -> bool {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return false,
    };

    let attributes = data.attributes.borrow();

    attributes.get(LocalName::from(attr_name)).is_some()
}

pub fn get_attr(el: &NodeRef, attr_name: &str) -> Option<String> {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return None,
    };

    let attributes = data.attributes.borrow();

    attributes
        .get(LocalName::from(attr_name))
        .map(ToOwned::to_owned)
}

pub fn insert_attr<T: Into<String>>(el: &NodeRef, attr_name: &str, attr_val: T) {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return,
    };

    let mut attributes = data.attributes.borrow_mut();

    attributes.insert(LocalName::from(attr_name), attr_val.into());
}

pub fn remove_attr(el: &NodeRef, attr_name: &str) {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return,
    };

    let mut attributes = data.attributes.borrow_mut();

    attributes.remove(LocalName::from(attr_name));
}

pub fn has_class(el: &NodeRef, class: &str) -> bool {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return false,
    };

    let attributes = data.attributes.borrow();

    if let Some(class_attr) = attributes.get(local_name!("class")) {
        class_attr.split_whitespace().any(|piece| piece == class)
    } else {
        false
    }
}

pub fn add_class(el: &NodeRef, class: &str) {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return,
    };

    let mut attributes = data.attributes.borrow_mut();

    let new_class_attr = if let Some(old_class_attr) = attributes.get(local_name!("class")) {
        if old_class_attr
            .split_whitespace()
            .any(|piece| piece == class)
        {
            old_class_attr.to_owned()
        } else {
            old_class_attr.to_owned() + " " + class
        }
    } else {
        class.to_owned()
    };

    attributes.insert(local_name!("class"), new_class_attr);
}

// Get the content of the text node wrapped by the given node.
pub fn get_text_content(el: &NodeRef) -> String {
    for child in el.children() {
        if let Some(content) = child.as_text() {
            let content = content.borrow().trim().to_owned();

            if !content.is_empty() {
                return content;
            }
        }
    }
    "".to_owned()
}

fn is_valid(el: &NodeRef) -> bool {
    match el.as_text() {
        Some(text) => !text.borrow().trim().is_empty(),
        None => true,
    }
}

// If the node only has one child, extract it.
pub fn get_only_child(el: &NodeRef) -> Option<NodeRef> {
    let children = el
        .children()
        .filter(|el| is_valid(el))
        .collect::<Vec<NodeRef>>();

    if children.len() == 1 {
        Some(children[0].clone())
    } else {
        None
    }
}

// Copy the content of a node to another node.
pub fn copy_content(from_el: &NodeRef, to_el: &NodeRef) {
    for from_el_child in from_el.children().filter(|el| is_valid(el)) {
        to_el.append(from_el_child.clone());
    }
}

pub fn deep_clone(el: &NodeRef) -> NodeRef {
    let root = NodeRef::new(el.data().clone());

    for child in el.children() {
        root.append(deep_clone(&child));
    }

    root
}

pub fn escape_html<T: Into<String>>(text: T) -> String {
    text.into().replace("&", "&amp;").replace("<", "&lt;")
}

pub fn select_first(el: &NodeRef, selectors: &str) -> Option<NodeRef> {
    match el.select_first(selectors) {
        Ok(el_data) => Some(el_data.as_node().clone()),
        _ => None,
    }
}

pub fn select(el: &NodeRef, selectors: &str) -> impl Iterator<Item = NodeRef> {
    el.select(selectors)
        .expect("selectors should be valid")
        .map(|el_data| el_data.as_node().clone())
}

fn scoping_nodes<F>(el: &NodeRef, filter_fn: F) -> impl Iterator<Item = NodeRef>
where
    F: Copy + Fn(&NodeRef) -> bool,
{
    el.inclusive_preceding_siblings().filter(filter_fn).chain(
        el.ancestors().flat_map(move |ancestor_el| {
            ancestor_el.inclusive_preceding_siblings().filter(filter_fn)
        }),
    )
}

fn relevant_heading(el: &NodeRef) -> Option<NodeRef> {
    lazy_static! {
        // heading level tags
        static ref HEADING_LEVEL_TAGS: HashSet<&'static str> = hashset! {
            "h1", "h2", "h3", "h4", "h5", "h6"
        };
    }

    scoping_nodes(el, |scope_el| match get_tag(scope_el) {
        Some(tag) => HEADING_LEVEL_TAGS.contains(tag.as_str()),
        None => false,
    })
    .next()
}

// Get the name of the nearest section to the node.
pub fn get_section(el: &NodeRef) -> Option<String> {
    match relevant_heading(el) {
        Some(heading_el) => {
            if has_class(&heading_el, "no-ref") {
                None
            } else {
                Some(get_text_content(&heading_el))
            }
        }
        None => Some("Unnamed section".to_owned()),
    }
}

pub fn get_closest_attr<'a>(el: &NodeRef, attr_names: &[&'a str]) -> Option<Attr<'a>> {
    for ancestor_el in el.inclusive_ancestors() {
        for attr_name in attr_names {
            if let Some(attr_val) = get_attr(&ancestor_el, attr_name) {
                return Some((attr_name, attr_val));
            }
        }
    }
    None
}

pub fn has_ancestor(el: &NodeRef, filter_fn: impl Fn(&NodeRef) -> bool) -> bool {
    el.ancestors().any(|ancestor_el| filter_fn(&ancestor_el))
}

pub fn dedup_ids(root: &NodeRef) {
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

    for el in select(root, "[id]") {
        let id = get_attr(&el, "id").unwrap();
        ids.entry(id).or_default().push(el);
    }

    for (id, els) in ids {
        if els.len() <= 1 {
            continue;
        }

        for (i, el) in els.iter().enumerate().skip(1) {
            let new_id = format!("{}{}", id, to_circled_digits(i));
            insert_attr(el, "id", new_id);
        }
    }
}
