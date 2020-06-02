use kuchiki::{Attribute, ExpandedName, NodeRef};
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

pub fn new_style(text: &str) -> NodeRef {
    let el = new_element("style", None);
    el.append(NodeRef::new_text(text));
    el
}

pub fn replace_node(old_node: &NodeRef, new_node: &NodeRef) {
    old_node.insert_before(new_node.clone());
    old_node.detach();
}
