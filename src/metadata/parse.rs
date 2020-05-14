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

#[derive(Debug, Clone, Default)]
pub struct Editor {
    pub name: String,
    pub w3c_id: Option<String>,
    pub org: Option<String>,
    pub org_link: Option<String>,
    pub link: Option<String>,
    pub email: Option<String>,
}

impl Editor {
    pub fn new(name: String) -> Self {
        Editor {
            name,
            ..Default::default()
        }
    }
}

pub fn parse_editor(val: &str) -> Result<Vec<Editor>, &'static str> {
    // <editor> := <editor-name> <option>
    // <option> := [<affiliation-name>] [<email> | <link> | (<email> <link>) | (<link> <email>)]

    lazy_static! {
        // w3c id reg
        static ref W3C_ID_REG: Regex = Regex::new(r"w3cid \d+$").unwrap();
        // link reg
        static ref LINK_REG: Regex = Regex::new(r"^\w+:").unwrap();
        // email reg
        static ref EMAIL_REG: Regex = Regex::new(r".+@.+\..+").unwrap();
    }

    fn is_linkish(piece: &str) -> bool {
        LINK_REG.is_match(piece)
    }

    fn is_emailish(piece: &str) -> bool {
        EMAIL_REG.is_match(piece)
    }

    // TODO: unescape pieces
    let mut pieces = val
        .split(",")
        .map(|piece| piece.trim())
        .collect::<Vec<&str>>();

    if pieces.is_empty() {
        return Err("invalid editor");
    }

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

    // handle ambiguous pieces
    if pieces.len() == 3
        && ((is_emailish(pieces[1]) && is_linkish(pieces[2]))
            || (is_linkish(pieces[1]) && is_emailish(pieces[2])))
    {
        // [org, email, link] or [org, link, email]
        editor.org = Some(pieces[0].to_owned());
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
        editor.org = Some(pieces[0].to_owned());
        if is_emailish(pieces[1]) {
            editor.email = Some(pieces[1].to_owned());
        } else {
            editor.link = Some(pieces[1].to_owned());
        }
    } else if pieces.len() == 1 {
        // [org], [email], or [url]
        if is_emailish(pieces[0]) {
            editor.email = Some(pieces[0].to_owned());
        } else if is_linkish(pieces[0]) {
            editor.link = Some(pieces[0].to_owned());
        } else {
            editor.org = Some(pieces[0].to_owned());
        }
    } else if pieces.len() != 0 {
        return Err("wrong format");
    }

    // check if the org ends with an email or a link
    if let Some(org) = editor.org.clone() {
        if org.contains(" ") {
            let org_pieces = org
                .split(" ")
                .map(|org_piece| org_piece.trim())
                .collect::<Vec<_>>();

            let last_org_piece = org_pieces.last().unwrap();

            if is_emailish(last_org_piece) || is_linkish(last_org_piece) {
                editor.org = Some(org_pieces[..org_pieces.len() - 1].join(" "));
                editor.org_link = Some(last_org_piece.to_string());
            }
        }
    }

    Ok(vec![editor])
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
