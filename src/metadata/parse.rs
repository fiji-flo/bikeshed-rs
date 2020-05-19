use regex::Regex;

use crate::util::date::{Date, ParseResult};

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
        return Err("wrong format");
    }

    // check if the org ends with an email or a link
    if let Some(org) = org {
        if org.contains(" ") {
            let org_pieces = org
                .split(" ")
                .map(|org_piece| org_piece.trim())
                .collect::<Vec<_>>();

            let last_org_piece = org_pieces.last().unwrap();

            if is_emailish(last_org_piece) || is_linkish(last_org_piece) {
                editor.org = Some(org_pieces[..org_pieces.len() - 1].join(" "));
                editor.org_link = Some(last_org_piece.to_string());
            } else {
                editor.org = Some(org_pieces.join(" "));
            }
        } else {
            editor.org = Some(org);
        }
    }

    Ok(editor)
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
    use super::{parse_editor, Editor};
    use std::collections::BTreeMap;

    #[test]
    fn test_parse_editor() {
        let cases: BTreeMap<&'static str, Result<Editor, &'static str>> = btreemap! {
            // empty value
            "" => Err("invalid editor"),
            // name
            "Mario" => Ok(Editor::new("Mario".to_owned())),
            // name, w3cid
            "Mario, w3cid 12345" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                ..Default::default()
            }),
            // [org]
            "Mario, w3cid 12345, Nintendo" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                ..Default::default()
            }),
            // [email]
            "Mario, w3cid 12345, hi@mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                ..Default::default()
            }),
            // [link]
            "Mario, w3cid 12345, https://mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [email, link]
            "Mario, w3cid 12345, hi@mario.com, https://mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [link, email]
            "Mario, w3cid 12345, https://mario.com, hi@mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, email]
            "Mario, w3cid 12345, Nintendo, hi@mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, link]
            "Mario, w3cid 12345, Nintendo, https://mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, email, link]
            "Mario, w3cid 12345, Nintendo, hi@mario.com, https://mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // [org, link, email]
            "Mario, w3cid 12345, Nintendo, https://mario.com, hi@mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // w3cid @ index 2
            "Mario, Nintendo, w3cid 12345, https://mario.com, hi@mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // w3cid @ index 3
            "Mario, Nintendo, https://mario.com, w3cid 12345, hi@mario.com" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // w3cid @ index 4
            "Mario, Nintendo, https://mario.com, hi@mario.com, w3cid 12345" => Ok(Editor {
                name: "Mario".to_owned(),
                w3c_id: Some("12345".to_owned()),
                org: Some("Nintendo".to_owned()),
                email: Some("hi@mario.com".to_owned()),
                link: Some("https://mario.com".to_owned()),
                ..Default::default()
            }),
            // org ends with an email
            "Mario, Nintendo hi@nintendo.com" => Ok(Editor {
                name: "Mario".to_owned(),
                org: Some("Nintendo".to_owned()),
                org_link: Some("hi@nintendo.com".to_owned()),
                ..Default::default()
            }),
            // org ends with a link
            "Mario, Nintendo https://nintendo.com" => Ok(Editor {
                name: "Mario".to_owned(),
                org: Some("Nintendo".to_owned()),
                org_link: Some("https://nintendo.com".to_owned()),
                ..Default::default()
            }),
            // org ends without an email or a link
            "Tommy Vercetti, Rockstar Games" => Ok(Editor {
                name: "Tommy Vercetti".to_owned(),
                org: Some("Rockstar Games".to_owned()),
                ..Default::default()
            }),
            // wrong format
            "Mario, Nintendo, error, https://mario.com, hi@mario.com" => Err("wrong format"),
            "Mario, Nintendo, https://mario.com, error, hi@mario.com" => Err("wrong format"),
            "Mario, Nintendo, https://mario.com, hi@mario.com, error" => Err("wrong format"),
        };

        for (val, target) in cases {
            let result = parse_editor(val).map_err(|e| e);
            assert_eq!(result, target);
        }
    }
}
