use kuchiki::{Attribute, ExpandedName, NodeData, NodeRef};
use markup5ever::{LocalName, QualName};

pub type Attr = (&'static str, &'static str);

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

pub fn insert_attr<T: Into<String>>(el: &NodeRef, attr_name: &str, attr_val: T) {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return,
    };

    let mut attributes = data.attributes.borrow_mut();
    attributes.insert(LocalName::from(attr_name), attr_val.into());
}

pub fn get_attr(el: &NodeRef, attr_name: &str) -> Option<String> {
    let data = match el.data() {
        NodeData::Element(data) => data,
        _ => return None,
    };

    let attributes = data.attributes.borrow();

    match attributes.get(LocalName::from(attr_name)) {
        Some(attr) => Some(attr.to_owned()),
        None => None,
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
