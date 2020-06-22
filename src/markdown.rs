use regex::Regex;

// Get HTML lines.
pub fn parse(lines: &[String]) -> Vec<String> {
    let tokens = tokenize_lines(lines);
    parse_tokens(&tokens)
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Blank,
    EqualsLine,
    Block,
    Text,
    End,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    line: String,
}

impl Token {
    fn new<T: Into<String>>(kind: TokenKind, line: T) -> Self {
        Token {
            kind,
            line: line.into(),
        }
    }

    fn new_blank() -> Self {
        Self::new(TokenKind::Blank, "")
    }

    fn new_end() -> Self {
        Self::new(TokenKind::End, "")
    }
}

#[derive(Debug)]
struct TokenStream<'a> {
    tokens: &'a [Token],
    curr: usize,
    // the token before all given tokens
    before: Token,
    // the token after all given tokens
    after: Token,
}

impl<'a> TokenStream<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        TokenStream {
            tokens,
            curr: 0,
            before: Token::new_blank(),
            after: Token::new_end(),
        }
    }

    fn move_to_next(&mut self) {
        if self.curr < self.tokens.len() {
            self.curr += 1;
        }
    }

    fn curr(&self) -> &Token {
        if self.curr >= self.tokens.len() {
            &self.after
        } else {
            &self.tokens[self.curr]
        }
    }

    fn prev(&self) -> &Token {
        if self.curr == 0 {
            &self.before
        } else {
            &self.tokens[self.curr - 1]
        }
    }

    #[allow(dead_code)]
    fn next(&self) -> &Token {
        if self.curr >= self.tokens.len() - 1 {
            &self.after
        } else {
            &self.tokens[self.curr + 1]
        }
    }
}

// Turn lines of text into block tokens, which'll be turned into MD blocks later.
fn tokenize_lines(lines: &[String]) -> Vec<Token> {
    lazy_static! {
        // regex for equals line
        static ref EQUALS_LINE_REG: Regex = Regex::new(r"={3,}\s*$").unwrap();
        // regex for html block
        static ref HTML_BLOCK_REG: Regex = Regex::new(r"<").unwrap();
    }

    let mut tokens = Vec::new();

    for line in lines.iter() {
        let token = if line.is_empty() {
            // blank
            Token::new(TokenKind::Blank, line)
        } else if EQUALS_LINE_REG.is_match(line) {
            // equals line
            Token::new(TokenKind::EqualsLine, line)
        } else if HTML_BLOCK_REG.is_match(line) {
            // block
            Token::new(TokenKind::Block, line)
        } else {
            // text
            Token::new(TokenKind::Text, line)
        };

        tokens.push(token);
    }

    tokens
}

fn parse_tokens(tokens: &[Token]) -> Vec<String> {
    let mut stream = TokenStream::new(tokens);
    let mut lines = Vec::new();

    loop {
        if stream.curr().kind == TokenKind::End {
            break;
        }

        match stream.curr().kind {
            TokenKind::Block => {
                lines.push(stream.curr().line.clone());
            }
            TokenKind::Text => {
                if stream.prev().kind == TokenKind::Blank {
                    lines.extend(parse_paragraph(&mut stream));
                }
            }
            _ => {}
        }

        stream.move_to_next();
    }

    lines
}

fn parse_paragraph(stream: &mut TokenStream) -> Vec<String> {
    let mut lines = vec![format!("<p>{}\n", stream.curr().line)];

    loop {
        stream.move_to_next();

        if stream.curr().kind == TokenKind::Text {
            lines.push(stream.curr().line.clone());
        } else {
            // append the end tag to the last line
            let last_index = lines.len() - 1;
            let last_line = lines.get_mut(last_index).unwrap();
            *last_line = format!("{}</p>\n", last_line.trim_end());
            break;
        }
    }

    lines
}
