pub mod comment;
mod token;

use regex::Regex;

use crate::config::INLINE_ELEMENT_TAGS;
use crate::util;
use token::*;

// Get HTML lines.
pub fn parse(lines: &[String], tab_size: u32) -> Vec<String> {
    let tokens = tokenize_lines(lines, tab_size);
    parse_tokens(&tokens, tab_size)
}

lazy_static! {
    // regex for opaque element
    static ref OPAQUE_REG: Regex = Regex::new(r"^\s*<(?P<tag>pre|xmp|script|style)[ >]").unwrap();
    // regex for fenced line
    static ref FENCED_LINE_REG: Regex = Regex::new(
        r"(?x)
        ^(\s*)
        (?P<tag>`{3,}|~{3,})
        ([^`]*)$"
    )
    .unwrap();
    // regex for equals line
    static ref EQUALS_LINE_REG: Regex = Regex::new(r"^={3,}\s*$").unwrap();
    // regex for dash line
    static ref DASH_LINE_REG: Regex = Regex::new(r"^-{3,}\s*$").unwrap();
    // regex for horizontal rule
    static ref HORIZONTAL_RULE_REG: Regex = Regex::new(
        r"(?x)
        ^((\*\s*){3,})$
        |^((-\s*){3,})$
        |^((_\s*){3,})$"
    )
    .unwrap();
    // regex for heading
    static ref HEADING_REG: Regex = Regex::new(
        r"(?x)
        ^(?P<prefix>\#{1,5})
        \s+
        (?P<text>[^\#]+)
        (
            (?P<another_prefix>\#{1,5})
            \s*
            \{\#
            (?P<id>[^}]+)
            \}
        )?
        \s*$"
    )
    .unwrap();
    // regex for numbered item
    static ref NUMBERED_REG: Regex = Regex::new(
        r"(?x)
        ^\s*
        (?P<num>-?[0-9]+)
        \.
        (\s+(?P<text>.*)|$)"
    )
    .unwrap();
    // regex for bulleted item
    static ref BULLETED_REG: Regex = Regex::new(
        r"(?x)
        ^\s*
        [*+-]
        (\s+(?P<text>.*)|$)"
    )
    .unwrap();
    // regex for definition item
    static ref DEF_REG: Regex = Regex::new(
        r"(?x)
        ^\s*
        (?P<prefix>:{1,2})
        (\s+(?P<text>.*)|$)"
    )
    .unwrap();
    // regex for quote block
    static ref QUOTE_BLOCK_REG: Regex = Regex::new(r"^\s*>\s?(?P<text>.*)").unwrap();
    // regex for markup block
    static ref MARKUP_BLOCK_REG: Regex = Regex::new(r"^\s*</?(?P<tag>[\w-]+)").unwrap();
}

fn is_single_line_heading(line: &str) -> bool {
    let caps = match HEADING_REG.captures(line) {
        Some(caps) => caps,
        None => return false,
    };

    let another_prefix = match caps.name("another_prefix") {
        Some(another_prefix) => another_prefix,
        None => return true,
    };

    let left_prefix = &caps["prefix"];
    left_prefix.len() == another_prefix.as_str().len()
}

fn extract_def_token_kind(line: &str) -> Option<TokenKind> {
    let caps = DEF_REG.captures(line)?;

    if caps["prefix"].len() == 1 {
        Some(TokenKind::Dt)
    } else {
        Some(TokenKind::Dd)
    }
}

fn starts_with_inline_element(line: &str) -> bool {
    let caps = MARKUP_BLOCK_REG.captures(line).unwrap();
    let tag = &caps["tag"];
    INLINE_ELEMENT_TAGS.contains(tag)
}

// Turn lines of text into block tokens, which'll be turned into MD blocks later.
fn tokenize_lines(lines: &[String], tab_size: u32) -> Vec<Token> {
    let make_token = |kind: TokenKind, line: &str| -> Token {
        match kind {
            TokenKind::Blank => Token::new_blank(),
            TokenKind::End => Token::new_end(),
            _ => {
                let indent_level = util::indent::get_indent_level(&line, tab_size);
                Token::new(kind, line, indent_level)
            }
        }
    };

    let mut tokens = Vec::new();
    let mut raw_token_stack: Vec<RawToken> = Vec::new();

    for line in lines.iter() {
        // Three kinds of "raw" elements, which prevent markdown processing inside of them.
        // 1. <pre>, which can contain markup and so can nest.
        // 2. <xmp>, <script>, and <style>, which contain raw text, can't nest.
        // 3. Markdown code blocks, which contain raw text, can't nest.

        if let Some(top_raw_token) = raw_token_stack.last() {
            if top_raw_token.kind == RawTokenKind::Element && line.contains(&top_raw_token.tag) {
                // opaque element
                raw_token_stack.pop();
                tokens.push(Token::new_raw(line));
                continue;
            }

            if top_raw_token.kind == RawTokenKind::Fenced
                && FENCED_LINE_REG.is_match(&line)
                && line[0..1] == top_raw_token.tag[0..1]
                && line.len() >= top_raw_token.tag.len()
            {
                // end fenced line
                raw_token_stack.pop();
                tokens.push(Token::new_raw("</pre>"));
                continue;
            }

            if !top_raw_token.is_nestable {
                // an internal line (but for the no-nesting elements)
                tokens.push(Token::new_raw(line));
                continue;
            }
        }

        // Handle opaque elements.
        if let Some(caps) = OPAQUE_REG.captures(&line) {
            tokens.push(make_token(TokenKind::Raw, line));

            let element_tag = &caps["tag"];

            if !line.contains(&format!("</{}>", element_tag)) {
                // The start tag and the end tag are not in the same line.
                let is_nestable = match element_tag {
                    "pre" => true,
                    "xmp" | "script" | "style" => false,
                    _ => die!("Unknown opaque element tag: {}.", element_tag),
                };

                raw_token_stack.push(RawToken {
                    kind: RawTokenKind::Element,
                    tag: element_tag.to_owned(),
                    is_nestable,
                });
            }

            continue;
        }

        // Handle fenced line.
        if let Some(caps) = FENCED_LINE_REG.captures(&line) {
            tokens.push(make_token(TokenKind::Raw, "<pre>"));

            let frenced_tag = &caps["tag"];

            raw_token_stack.push(RawToken {
                kind: RawTokenKind::Fenced,
                tag: frenced_tag.to_owned(),
                is_nestable: false,
            });

            continue;
        }

        if !raw_token_stack.is_empty() {
            tokens.push(make_token(TokenKind::Raw, line));
            continue;
        }

        // Handle other tokens.
        let token = if line.is_empty() {
            // blank
            make_token(TokenKind::Blank, &line)
        } else if EQUALS_LINE_REG.is_match(&line) {
            // equals line
            make_token(TokenKind::EqualsLine, &line)
        } else if DASH_LINE_REG.is_match(&line) {
            // dash line
            make_token(TokenKind::DashLine, &line)
        } else if HORIZONTAL_RULE_REG.is_match(&line) {
            // horizontal rule
            make_token(TokenKind::HorizontalRule, &line)
        } else if is_single_line_heading(&line) {
            // single line heading
            make_token(TokenKind::Head, &line)
        } else if NUMBERED_REG.is_match(&line) {
            // numbered item
            make_token(TokenKind::Numbered, &line)
        } else if BULLETED_REG.is_match(&line) {
            // bulleted item
            make_token(TokenKind::Bulleted, &line)
        } else if let Some(token_kind) = extract_def_token_kind(&line) {
            // definition item
            make_token(token_kind, &line)
        } else if QUOTE_BLOCK_REG.is_match(&line) {
            // quote block
            make_token(TokenKind::QuoteBlock, &line)
        } else if MARKUP_BLOCK_REG.is_match(&line) {
            if starts_with_inline_element(&line) {
                // text
                make_token(TokenKind::Text, &line)
            } else {
                // markup block
                make_token(TokenKind::MarkupBlock, &line)
            }
        } else {
            // text
            make_token(TokenKind::Text, &line)
        };

        tokens.push(token);
    }

    tokens
}

fn parse_tokens(tokens: &[Token], tab_size: u32) -> Vec<String> {
    let mut stream = TokenStream::new(tokens, tab_size);
    let mut lines = Vec::new();

    loop {
        match stream.curr().kind {
            TokenKind::End => break,
            TokenKind::Raw | TokenKind::MarkupBlock => {
                lines.push(stream.curr().line.clone());
            }
            TokenKind::Head => {
                lines.push(parse_single_line_heading(&mut stream));
            }
            TokenKind::Text => {
                if stream.next().kind == TokenKind::EqualsLine
                    || stream.next().kind == TokenKind::DashLine
                {
                    lines.push(parse_multi_line_heading(&mut stream));
                } else if stream.prev().kind == TokenKind::Blank {
                    lines.extend(parse_paragraph(&mut stream));
                } else {
                    lines.push(stream.curr().line.clone());
                }
            }
            TokenKind::HorizontalRule | TokenKind::DashLine => {
                lines.push(make_horizontal_rule());
            }
            TokenKind::Numbered | TokenKind::Bulleted | TokenKind::Dt | TokenKind::Dd => {
                lines.extend(parse_list(&mut stream));
            }
            TokenKind::QuoteBlock => {
                lines.extend(parse_quote_block(&mut stream));
            }
            _ => {
                lines.push(stream.curr().line.clone());
            }
        }

        stream.advance();
    }

    lines
}

#[inline]
fn make_horizontal_rule() -> String {
    "<hr>".to_owned()
}

// NOTE: When a particular section-parsing function is over, the current token
// in the stream should be the last token of the section.

fn parse_single_line_heading(stream: &mut TokenStream) -> String {
    let caps = HEADING_REG.captures(&stream.curr().line).unwrap();

    let prefix = &caps["prefix"];
    let level = prefix.len() + 1;

    let text = &caps["text"].trim();

    let id_attr = if let Some(id) = caps.name("id") {
        format!("id = {}", id.as_str())
    } else {
        String::new()
    };

    format!(
        "<h{level} {id_attr}>{text}</h{level}>\n",
        level = level,
        id_attr = id_attr,
        text = text
    )
}

fn parse_multi_line_heading(stream: &mut TokenStream) -> String {
    lazy_static! {
        // regex for text with id
        static ref TEXT_WITH_ID_REG: Regex = Regex::new(
            r"(?x)
            (?P<text>.*)
            \s*
            \{\s*\#
            (?P<id>[^}]+)
            \s*\}
            \s*$"
        )
        .unwrap();
    }

    let level = match stream.next().kind {
        TokenKind::EqualsLine => 2,
        TokenKind::DashLine => 3,
        _ => die!(
            "[Markdown] Fail to parse a multi-line heading from:\n{}\n{}",
            stream.curr().line,
            stream.next().line
        ),
    };

    let (text, id_attr) = if let Some(caps) = TEXT_WITH_ID_REG.captures(&stream.curr().line) {
        (
            caps["text"].trim().to_owned(),
            format!("id = {}", &caps["id"]),
        )
    } else {
        (stream.curr().line.trim().to_owned(), String::new())
    };

    let heading = format!(
        "<h{level} {id_attr}>{text}</h{level}>\n",
        level = level,
        id_attr = id_attr,
        text = text
    );

    stream.advance();

    heading
}

fn parse_paragraph(stream: &mut TokenStream) -> Vec<String> {
    let mut lines = vec![format!("<p>{}", stream.curr().line)];

    loop {
        if stream.next().kind == TokenKind::Text {
            stream.advance();
            lines.push(stream.curr().line.clone());
        } else {
            // Append the end tag to the last line.
            let last_index = lines.len() - 1;
            let last_line = lines.get_mut(last_index).unwrap();
            *last_line = format!("{}</p>\n", last_line.trim_end());
            break;
        }
    }

    lines
}

fn parse_list(stream: &mut TokenStream) -> Vec<String> {
    let (target_tokens, reg, outer_tag): (Vec<TokenKind>, &Regex, &str) = match stream.curr().kind {
        TokenKind::Numbered => (vec![TokenKind::Numbered], &NUMBERED_REG, "ol"),
        TokenKind::Bulleted => (vec![TokenKind::Bulleted], &BULLETED_REG, "ul"),
        TokenKind::Dt | TokenKind::Dd => (vec![TokenKind::Dt, TokenKind::Dd], &DEF_REG, "dl"),
        _ => die!("[Markdown] Try to parse a line that isn't a list."),
    };

    let outer_el_attr = match stream.curr().kind {
        TokenKind::Numbered => {
            let caps = reg.captures(&stream.curr().line).unwrap();
            let start_num = caps["num"].parse::<i32>().unwrap();

            if start_num == 1 {
                "data-md".to_owned()
            } else {
                format!("data-md start={}", start_num)
            }
        }
        _ => "data-md".to_owned(),
    };

    let top_indent_level = stream.curr().indent_level;

    let parse_item = |stream: &mut TokenStream| -> (TokenKind, Vec<String>) {
        let mut lines = Vec::new();

        let token_kind = stream.curr().kind;

        if let Some(caps) = reg.captures(&stream.curr().line) {
            let text = caps.name("text").map_or("", |m| m.as_str());
            lines.push(text.to_owned());
        }

        loop {
            // Break the loop if we reach the end of this item.
            if stream.next().kind == TokenKind::End {
                break;
            }
            if target_tokens.contains(&stream.next().kind)
                && stream.next().indent_level == top_indent_level
            {
                break;
            }
            if stream.next().indent_level < top_indent_level {
                break;
            }
            if stream.next().kind == TokenKind::Blank
                && !target_tokens.contains(&stream.next_next().kind)
                && stream.next_next().indent_level <= top_indent_level
            {
                break;
            }

            stream.advance();

            lines.push(util::indent::trim_indent(
                &stream.curr().line,
                top_indent_level + 1,
                stream.tab_size(),
            ));
        }

        (token_kind, lines)
    };

    let mut lines = vec![format!("<{} {}>", outer_tag, outer_el_attr)];

    loop {
        let (token_kind, item_lines) = parse_item(stream);
        let tag = match token_kind {
            TokenKind::Numbered | TokenKind::Bulleted => "li",
            TokenKind::Dt => "dt",
            _ => "dd",
        };

        // Generate an item.
        lines.push(format!("<{} data-md>", tag));
        lines.extend(parse(&item_lines, stream.tab_size()));
        lines.push(format!("</{}>", tag));

        // Break the loop if we reach the end of this list.
        if stream.next().kind == TokenKind::End {
            break;
        }
        if stream.next().indent_level < top_indent_level {
            break;
        }
        if stream.next().kind == TokenKind::Blank
            && !target_tokens.contains(&stream.next_next().kind)
            && stream.next_next().indent_level <= top_indent_level
        {
            break;
        }

        if stream.next().kind == TokenKind::Blank {
            stream.advance();
        }

        stream.advance();
    }

    lines.push(format!("</{}>", outer_tag));

    lines
}

fn parse_quote_block(stream: &mut TokenStream) -> Vec<String> {
    let extract_text_from_quote_block = |line: &str| -> Option<String> {
        let caps = QUOTE_BLOCK_REG.captures(line).unwrap();

        if let Some(text) = caps.name("text") {
            Some(text.as_str().to_owned())
        } else {
            None
        }
    };

    let mut inner_lines = Vec::new();

    if let Some(text) = extract_text_from_quote_block(&stream.curr().line) {
        inner_lines.push(text);
    }

    let top_indent_level = stream.curr().indent_level;

    loop {
        if stream.next().indent_level < top_indent_level {
            break;
        }

        match stream.next().kind {
            TokenKind::QuoteBlock => {
                stream.advance();
                if let Some(text) = extract_text_from_quote_block(&stream.curr().line) {
                    inner_lines.push(text);
                }
            }
            TokenKind::Text => {
                stream.advance();
                inner_lines.push(stream.curr().line.to_owned());
            }
            _ => break,
        }
    }

    let mut lines = Vec::new();

    lines.push("<blockquote>".to_owned());
    lines.extend(parse(&inner_lines, stream.tab_size()));
    lines.push("</blockquote>".to_owned());

    lines
}
