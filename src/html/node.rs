use kuchiki::{Attribute, ExpandedName, NodeData, NodeRef};
use markup5ever::{LocalName, QualName};

pub fn new_element<I>(name: &str, attributes: I) -> NodeRef
where
    I: IntoIterator<Item = (&'static str, String)>,
{
    NodeRef::new_element(
        QualName::new(None, ns!(html), LocalName::from(name)),
        attributes.into_iter().map(|(key, val)| {
            let expanded_name = ExpandedName::new(ns!(), LocalName::from(key));
            let attribute = Attribute {
                prefix: None,
                value: val,
            };
            (expanded_name, attribute)
        }),
    )
}

pub fn new_text<T: Into<String>>(text: T) -> NodeRef {
    kuchiki::NodeRef::new_text(text)
}

pub fn new_style<T: Into<String>>(text: T) -> NodeRef {
    let el = new_element("style", None);
    el.append(NodeRef::new_text(text));
    el
}

pub fn new_a<I, T>(attributes: I, text: T) -> NodeRef
where
    I: IntoIterator<Item = (&'static str, String)>,
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

pub fn add_class(el: &NodeRef, class: &str) {
    if let NodeData::Element(el_data) = el.data() {
        let ref mut attributes = el_data.attributes.borrow_mut();

        let new_class = if let Some(old_class) = attributes.get(local_name!("class")) {
            old_class.to_owned() + " " + class
        } else {
            class.to_owned()
        };

        attributes.insert(local_name!("class"), new_class);
    }
}
