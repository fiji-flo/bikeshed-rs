use regex::Regex;

// Get HTML lines.
pub fn parse(lines: &[String]) -> Vec<String> {
    let tokens = tokenize_lines(lines);
    parse_tokens(&tokens)
}

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Blank,
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

// Turn lines of text into block tokens, which'll be turned into MD blocks later.
fn tokenize_lines(lines: &[String]) -> Vec<Token> {
    lazy_static! {
        // regex for html block
        static ref HTML_BLOCK_REG: Regex = Regex::new(r"<").unwrap();
    }

    let mut tokens = Vec::new();

    for line in lines.iter() {
        if line.is_empty() {
            tokens.push(Token::new(TokenKind::Blank, line));
        } else if HTML_BLOCK_REG.is_match(line) {
            tokens.push(Token::new(TokenKind::Block, line));
        } else {
            tokens.push(Token::new(TokenKind::Text, line));
        }
    }

    tokens
}

fn parse_tokens(tokens: &[Token]) -> Vec<String> {
    let mut lines = Vec::new();

    let mut prev_token = Token::new_blank();
    let mut stream = tokens.into_iter();

    loop {
        let curr_token = if let Some(token) = stream.next() {
            token
        } else {
            break;
        };

        match curr_token.kind {
            TokenKind::Blank => {
                prev_token = curr_token.clone();
            }
            TokenKind::Block => {
                lines.push(curr_token.line.clone());
                prev_token = curr_token.clone();
            }
            TokenKind::Text => {
                if prev_token.kind == TokenKind::Blank {
                    lines.extend(parse_paragraph(
                        &curr_token.line,
                        &mut stream,
                        &mut prev_token,
                    ));
                } else {
                    prev_token = curr_token.clone();
                }
            }
            _ => {}
        }
    }

    lines
}

fn parse_paragraph<'a, I>(text: &str, stream: &mut I, prev_token: &mut Token) -> Vec<String>
where
    I: Iterator<Item = &'a Token>,
{
    let mut lines = vec![format!("<p>{}\n", text)];

    loop {
        let curr_token = if let Some(token) = stream.next() {
            token.to_owned()
        } else {
            Token::new_end()
        };

        if curr_token.kind == TokenKind::Text {
            lines.push(curr_token.line);
        } else {
            *prev_token = curr_token.clone();
            // append the end tag to the last line
            let last_index = lines.len() - 1;
            let last_line = lines.get_mut(last_index).unwrap();
            *last_line = format!("{}</p>\n", last_line.trim_end());
            break;
        }
    }

    lines
}
