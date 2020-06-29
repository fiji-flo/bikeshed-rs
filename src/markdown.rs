use regex::Regex;

// Get HTML lines.
pub fn parse(lines: &[String], tab_size: u32) -> Vec<String> {
    let tokens = tokenize_lines(lines, tab_size);
    parse_tokens(&tokens, tab_size)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenKind {
    Blank,
    EqualsLine,
    DashLine,
    Head,
    Numbered,
    Bulleted,
    Dt,
    Dd,
    Block,
    Text,
    End,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    line: String,
    indent_level: u32,
}

impl Token {
    fn new<T: Into<String>>(kind: TokenKind, line: T, indent_level: u32) -> Self {
        Token {
            kind,
            line: line.into(),
            indent_level,
        }
    }

    fn new_blank() -> Self {
        Token::new(TokenKind::Blank, "", u32::max_value())
    }

    fn new_end() -> Self {
        Token::new(TokenKind::End, "", u32::max_value())
    }
}

fn get_indent_level(text: &str, tab_size: u32) -> u32 {
    let tab_size = tab_size as usize;
    let mut indent_level = 0;
    let mut curr: usize = 0;

    loop {
        if curr >= text.len() {
            break;
        }

        if &text[curr..curr + 1] == "\t" {
            indent_level += 1;
            curr += 1;
        } else if curr + tab_size <= text.len()
            && &text[curr..curr + tab_size] == " ".repeat(tab_size)
        {
            indent_level += 1;
            curr += tab_size;
        } else {
            break;
        }
    }

    indent_level
}

fn trim_indentation(text: &str, indent_level: u32, tab_size: u32) -> String {
    // allow empty line
    if text.trim().is_empty() {
        return text.to_owned();
    }

    let tab_size = tab_size as usize;
    let mut offset: usize = 0;

    for _ in 0..indent_level {
        if &text[offset..offset + 1] == "\t" {
            offset += 1;
        } else if offset + tab_size <= text.len()
            && &text[offset..offset + tab_size] == " ".repeat(tab_size)
        {
            offset += tab_size;
        } else {
            die!("[Markdown] \"{}\" isn't indented enough.", text);
        }
    }

    text[offset..].to_owned()
}

#[derive(Debug)]
struct TokenStream<'a> {
    tokens: &'a [Token],
    curr: usize,
    tab_size: u32,
    // the token before all given tokens
    before: Token,
    // the token after all given tokens
    after: Token,
}

impl<'a> TokenStream<'a> {
    fn new(tokens: &'a [Token], tab_size: u32) -> Self {
        TokenStream {
            tokens,
            curr: 0,
            tab_size,
            before: Token::new_blank(),
            after: Token::new_end(),
        }
    }

    fn advance(&mut self) {
        if self.curr < self.tokens.len() {
            self.curr += 1;
        }
    }

    fn nth(&self, index: usize) -> &Token {
        if index >= self.tokens.len() {
            &self.after
        } else {
            &self.tokens[index]
        }
    }

    fn curr(&self) -> &Token {
        self.nth(self.curr)
    }

    fn prev(&self) -> &Token {
        if self.curr == 0 {
            &self.before
        } else {
            &self.nth(self.curr - 1)
        }
    }

    fn next(&self) -> &Token {
        &self.nth(self.curr + 1)
    }

    fn next_next(&self) -> &Token {
        &self.nth(self.curr + 2)
    }
}

lazy_static! {
    // regex for equals line
    static ref EQUALS_LINE_REG: Regex = Regex::new(r"^={3,}\s*$").unwrap();
    // regex for dash line
    static ref DASH_LINE_REG: Regex = Regex::new(r"^-{3,}\s*$").unwrap();
    // regex for heading
    static ref HEADING_REG: Regex = Regex::new(r"^(?P<prefix>#{1,5})\s+(?P<text>[^#]+)((?P<another_prefix>#{1,5})\s*\{#(?P<id>[^}]+)\})?\s*$").unwrap();
    // regex for numbered item
    static ref NUMBERED_REG: Regex = Regex::new(r"^\s*(?P<id>[0-9]+)\.\s*(?P<text>.*)").unwrap();
    // regex for bulleted item
    static ref BULLETED_REG: Regex = Regex::new(r"^\s*[*+-]\s*(?P<text>.*)").unwrap();
    // regex for definition item
    static ref DEF_REG: Regex = Regex::new(r"^\s*(?P<prefix>:{1,2})\s*(?P<text>.*)").unwrap();
    // regex for html block
    static ref HTML_BLOCK_REG: Regex = Regex::new(r"<").unwrap();
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

    let left_prefix = caps.name("prefix").unwrap();
    left_prefix.as_str().len() == another_prefix.as_str().len()
}

fn extract_def_token_kind(line: &str) -> Option<TokenKind> {
    let caps = DEF_REG.captures(line)?;

    if caps.name("prefix").unwrap().as_str().len() == 1 {
        Some(TokenKind::Dt)
    } else {
        Some(TokenKind::Dd)
    }
}

// Turn lines of text into block tokens, which'll be turned into MD blocks later.
fn tokenize_lines(lines: &[String], tab_size: u32) -> Vec<Token> {
    let make_token = |kind: TokenKind, line: &str| -> Token {
        match kind {
            TokenKind::Blank => Token::new_blank(),
            TokenKind::End => Token::new_end(),
            _ => {
                let indent_level = get_indent_level(&line, tab_size);
                Token::new(kind, line, indent_level)
            }
        }
    };

    let mut tokens = Vec::new();

    for line in lines.iter() {
        let token = if line.is_empty() {
            // blank
            make_token(TokenKind::Blank, line)
        } else if EQUALS_LINE_REG.is_match(line) {
            // equals line
            make_token(TokenKind::EqualsLine, line)
        } else if DASH_LINE_REG.is_match(line) {
            // dash line
            make_token(TokenKind::DashLine, line)
        } else if is_single_line_heading(line) {
            // single line heading
            make_token(TokenKind::Head, line)
        } else if NUMBERED_REG.is_match(line) {
            // numbered item
            make_token(TokenKind::Numbered, line)
        } else if BULLETED_REG.is_match(line) {
            // bulleted item
            make_token(TokenKind::Bulleted, line)
        } else if let Some(token_kind) = extract_def_token_kind(line) {
            // definition item
            make_token(token_kind, line)
        } else if HTML_BLOCK_REG.is_match(line) {
            // block
            make_token(TokenKind::Block, line)
        } else {
            // text
            make_token(TokenKind::Text, line)
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
            TokenKind::Block => {
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
            TokenKind::Numbered | TokenKind::Bulleted | TokenKind::Dt | TokenKind::Dd => {
                lines.extend(parse_list(&mut stream));
            }
            _ => {
                lines.push(stream.curr().line.clone());
            }
        }

        stream.advance();
    }

    lines
}

// NOTE: When a particular section-parsing function is over, the current token
// in the stream should be the last token of the section.

fn parse_single_line_heading(stream: &mut TokenStream) -> String {
    let caps = HEADING_REG.captures(&stream.curr().line).unwrap();

    let prefix = caps.name("prefix").unwrap().as_str();
    let level = prefix.len() + 1;

    let text = caps.name("text").unwrap().as_str().trim();

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
        static ref TEXT_WITH_ID_REG: Regex = Regex::new(r"(?P<text>.*)\s*\{\s*#(?P<id>[^}]+)\s*\}\s*$").unwrap();
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
            caps.name("text").unwrap().as_str().trim(),
            format!("id = {}", caps.name("id").unwrap().as_str()),
        )
    } else {
        (stream.curr().line.as_str().trim(), String::new())
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

    let top_indent_level = stream.curr().indent_level;

    let parse_item = |stream: &mut TokenStream| -> (TokenKind, Vec<String>) {
        let mut lines = Vec::new();

        let token_kind = stream.curr().kind;

        if let Some(caps) = reg.captures(&stream.curr().line) {
            let text = caps.name("text").map_or("", |t| t.as_str());
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

            lines.push(trim_indentation(
                &stream.curr().line,
                top_indent_level + 1,
                stream.tab_size,
            ));
        }

        (token_kind, lines)
    };

    let mut lines = vec![format!("<{}>", outer_tag)];

    loop {
        let (token_kind, item_lines) = parse_item(stream);
        let tag = match token_kind {
            TokenKind::Numbered | TokenKind::Bulleted => "li",
            TokenKind::Dt => "dt",
            _ => "dd",
        };

        // Generate an item.
        lines.push(format!("<{} data-md>", tag));
        lines.extend(parse(&item_lines, stream.tab_size));
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
