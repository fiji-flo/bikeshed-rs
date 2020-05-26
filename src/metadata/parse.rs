use regex::Regex;

use crate::util::boolish::boolish_to_bool;
use crate::util::boolset::BoolSet;
use crate::util::date::{Date, ParseResult};

pub fn parse_boilerplate(val: &str) -> Result<BoolSet<String>, &'static str> {
    // <boilerplate> := <pair> ("," <pair>)*
    // <pair> := ("omit" <section>) | (<section> <boolish>)

    let mut boilerplate = BoolSet::<String>::new_with_default(true);

    for item in val.split(",").map(|item| item.trim()) {
        let pieces = item
            .split(" ")
            .map(|piece| piece.trim().to_lowercase())
            .collect::<Vec<String>>();

        if pieces.len() != 2 {
            return Err("wrong boilerplate piece format");
        }

        if pieces[0] == "omit" {
            // "omit" <section>
            boilerplate.insert(pieces[1].clone(), false);
        } else {
            // <section> <boolish>
            if let Some(on_off) = boolish_to_bool(&pieces[1]) {
                boilerplate.insert(pieces[0].clone(), on_off);
            } else {
                return Err("wrong boolish format");
            }
        }
    }

    Ok(boilerplate)
}

pub fn parse_date(val: &str) -> ParseResult {
    if val == "now" {
        Ok(Date::now())
    } else {
        match Date::parse_from_str(val, "%Y-%m-%d") {
            Ok(date) => Ok(date),
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Editor {
    pub name: String,
    pub w3c_id: Option<String>,
    pub org: Option<String>,
    pub org_link: Option<String>,
    pub email: Option<String>,
    pub link: Option<String>,
}

impl Editor {
    fn new(name: String) -> Self {
        Editor {
            name,
            ..Default::default()
        }
    }
}

pub fn parse_editor(val: &str) -> Result<Editor, &'static str> {
    // <editor> := <editor-name> <option>
    // <option> := [<affiliation-name>] [<email> | <link> | (<email> <link>) | (<link> <email>)]

    lazy_static! {
        // w3c id reg
        static ref W3C_ID_REG: Regex = Regex::new(r"w3cid \d+$").unwrap();
        // link reg
        static ref LINK_REG: Regex = Regex::new(r"^\w+:").unwrap();
        // email reg
        static ref EMAIL_REG: Regex = Regex::new(r"^\w+@.+\..+").unwrap();
    }

    fn is_linkish(piece: &str) -> bool {
        LINK_REG.is_match(piece)
    }

    fn is_emailish(piece: &str) -> bool {
        EMAIL_REG.is_match(piece)
    }

    if val.is_empty() {
        return Err("invalid editor");
    }

    // TODO: unescape pieces
    let mut pieces = val
        .split(",")
        .map(|piece| piece.trim())
        .collect::<Vec<&str>>();

    let mut editor = Editor::new(pieces[0].to_owned());

    // extract w3c id
    pieces = pieces[1..]
        .iter()
        .cloned()
        .filter(|piece| {
            if W3C_ID_REG.is_match(piece) && editor.w3c_id.is_none() {
                editor.w3c_id = Some((&piece[6..]).to_owned());
                false
            } else {
                true
            }
        })
        .collect::<Vec<&str>>();

    let mut org: Option<String> = None;

    // handle ambiguous pieces
    if pieces.len() == 3
        && ((is_emailish(pieces[1]) && is_linkish(pieces[2]))
            || (is_linkish(pieces[1]) && is_emailish(pieces[2])))
    {
        // [org, email, link] or [org, link, email]
        org = Some(pieces[0].to_owned());
        if is_emailish(pieces[1]) {
            editor.email = Some(pieces[1].to_owned());
            editor.link = Some(pieces[2].to_owned());
        } else {
            editor.link = Some(pieces[1].to_owned());
            editor.email = Some(pieces[2].to_owned());
        }
    } else if pieces.len() == 2
        && ((is_emailish(pieces[0]) && is_linkish(pieces[1]))
            || (is_linkish(pieces[0]) && is_emailish(pieces[1])))
    {
        // [email, link] or [link, email]
        if is_emailish(pieces[0]) {
            editor.email = Some(pieces[0].to_owned());
            editor.link = Some(pieces[1].to_owned());
        } else {
            editor.link = Some(pieces[0].to_owned());
            editor.email = Some(pieces[1].to_owned());
        }
    } else if pieces.len() == 2 && (is_emailish(pieces[1]) || is_linkish(pieces[1])) {
        // [org, email] or [org, link]
        org = Some(pieces[0].to_owned());
        if is_emailish(pieces[1]) {
            editor.email = Some(pieces[1].to_owned());
        } else {
            editor.link = Some(pieces[1].to_owned());
        }
    } else if pieces.len() == 1 {
        // [org], [email], or [link]
        if is_emailish(pieces[0]) {
            editor.email = Some(pieces[0].to_owned());
        } else if is_linkish(pieces[0]) {
            editor.link = Some(pieces[0].to_owned());
        } else {
            org = Some(pieces[0].to_owned());
        }
    } else if pieces.len() != 0 {
        return Err("wrong editor format");
    }

    // check if the org ends with an email or a link
    if let Some(org) = org {
        if org.contains(" ") {
            let org_pieces = org.rsplitn(2, ' ').collect::<Vec<&str>>();
            let left_part = org_pieces[1];
            let last_piece = org_pieces[0];

            if is_emailish(last_piece) || is_linkish(last_piece) {
                editor.org = Some(left_part.to_owned());
                editor.org_link = Some(last_piece.to_owned());
            } else {
                editor.org = Some([left_part, last_piece].join(" "));
            }
        } else {
            editor.org = Some(org);
        }
    }

    Ok(editor)
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorTerm {
    pub singular: String,
    pub plural: String,
}

impl EditorTerm {
    fn new(singular: String, plural: String) -> Self {
        EditorTerm { singular, plural }
    }
}

impl Default for EditorTerm {
    fn default() -> Self {
        EditorTerm::new("Editor".to_owned(), "Editors".to_owned())
    }
}

pub fn parse_editor_term(val: &str) -> Result<EditorTerm, &'static str> {
    // <editor-term> := <singular-term> "," <plural-term>

    let pieces = val
        .split(",")
        .map(|piece| piece.trim())
        .collect::<Vec<&str>>();

    if pieces.len() == 2 {
        Ok(EditorTerm::new(pieces[0].to_owned(), pieces[1].to_owned()))
    } else {
        Err("wrong editor term format")
    }
}

pub fn parse_level(val: &str) -> String {
    if val == "none" {
        String::new()
    } else {
        val.to_owned()
    }
}

pub fn parse_vec(val: &str) -> Vec<String> {
    vec![val.to_owned()]
}

#[cfg(test)]
mod tests {
    use super::{parse_boilerplate, parse_editor, parse_editor_term, Editor, EditorTerm};
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_boilerplate() {
        {
            let result = parse_boilerplate("");
            assert_eq!(result, Err("wrong boilerplate piece format"));
        }
        {
            let result = parse_boilerplate("omit logo");
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert_eq!(result.get("logo"), false);
                assert_eq!(result.get("copyright"), true);
            }
        }
        {
            let result = parse_boilerplate("logo omit");
            assert_eq!(result, Err("wrong boolish format"));
        }
        {
            let result = parse_boilerplate("omit logo, omit copyright");
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert_eq!(result.get("logo"), false);
                assert_eq!(result.get("copyright"), false);
                assert_eq!(result.get("warning"), true);
            }
        }
        {
            let result = parse_boilerplate("logo no");
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert_eq!(result.get("logo"), false);
                assert_eq!(result.get("copyright"), true);
            }
        }
        {
            let result = parse_boilerplate("logo logo");
            assert_eq!(result, Err("wrong boolish format"));
        }
        {
            let result = parse_boilerplate("logo no logo");
            assert_eq!(result, Err("wrong boilerplate piece format"));
        }
        {
            let result = parse_boilerplate("logo yes, omit copyright, warning no");
            assert!(result.is_ok());
            if let Ok(result) = result {
                assert_eq!(result.get("logo"), true);
                assert_eq!(result.get("copyright"), false);
                assert_eq!(result.get("warning"), false);
            }
        }
    }

    #[test]
    fn test_parse_editor() {
        let cases: BTreeMap<&'static str, Result<Editor, &'static str>> = btreemap! {
            // empty value
            "" => Err("invalid editor"),
            // name
            "Super Mario" => Ok(Editor::new("Super Mario".to_owned())),
            // name, w3cid
            "Super Mario, w3cid 12345" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                ..Default::default()
            }),
            // [org]
            "Super Mario, w3cid 12345, Nintendo" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                ..Default::default()
            }),
            // [email]
            "Super Mario, w3cid 12345, hi@mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                ..Default::default()
            }),
            // [link]
            "Super Mario, w3cid 12345, https://mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [email, link]
            "Super Mario, w3cid 12345, hi@mario.com, https://mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [link, email]
            "Super Mario, w3cid 12345, https://mario.com, hi@mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, email]
            "Super Mario, w3cid 12345, Nintendo, hi@mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, link]
            "Super Mario, w3cid 12345, Nintendo, https://mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, email, link]
            "Super Mario, w3cid 12345, Nintendo, hi@mario.com, https://mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, link, email]
            "Super Mario, w3cid 12345, Nintendo, https://mario.com, hi@mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // w3cid @ index 2
            "Super Mario, Nintendo, w3cid 12345, https://mario.com, hi@mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // w3cid @ index 3
            "Super Mario, Nintendo, https://mario.com, w3cid 12345, hi@mario.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // w3cid @ index 4
            "Super Mario, Nintendo, https://mario.com, hi@mario.com, w3cid 12345" => Ok(Editor {
                name: "Super Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // org ends with an email
            "Super Mario, Nintendo hi@nintendo.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                org: Some("Nintendo".to_owned()),
                org_link: Some("hi@nintendo.com".to_owned()),
                ..Default::default()
            }),
            // org ends with a link
            "Super Mario, Nintendo https://nintendo.com" => Ok(Editor {
                name: "Super Mario".to_owned(),
                org: Some("Nintendo".to_owned()),
                org_link: Some("https://nintendo.com".to_owned()),
                ..Default::default()
            }),
            // org ends without an email or a link
            "Super Mario, Nintendo not an email or a link" => Ok(Editor {
                name: "Super Mario".to_owned(),
                org: Some("Nintendo not an email or a link".to_owned()),
                ..Default::default()
            }),
            // wrong format
            "Super Mario, Nintendo, error, https://mario.com, hi@mario.com" => Err("wrong editor format"),
            "Super Mario, Nintendo, https://mario.com, error, hi@mario.com" => Err("wrong editor format"),
            "Super Mario, Nintendo, https://mario.com, hi@mario.com, error" => Err("wrong editor format"),
        };

        for (val, target) in cases {
            let result = parse_editor(val);
            assert_eq!(result, target);
        }
    }

    #[test]
    fn test_parse_editor_term() {
        let cases: BTreeMap<&'static str, Result<EditorTerm, &'static str>> = btreemap! {
            "x" => Err("wrong editor term format"),
            "x, xs" => Ok(EditorTerm::new("x".to_owned(), "xs".to_owned())),
            "x, xs, xss" => Err("wrong editor term format"),
        };

        for (val, target) in cases {
            let result = parse_editor_term(val);
            assert_eq!(result, target);
        }
    }
}
