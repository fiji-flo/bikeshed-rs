use regex::Regex;

// Get HTML lines.
pub fn parse(lines: &[String]) -> Vec<String> {
    let tokens = tokenize_lines(lines);
    parse_tokens(&tokens)
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Blank,
    Block(String),
    Text(String),
    End,
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
            tokens.push(Token::Blank);
        } else if HTML_BLOCK_REG.is_match(line) {
            tokens.push(Token::Block(line.clone()));
        } else {
            tokens.push(Token::Text(line.clone()));
        }
    }

    tokens
}

fn parse_tokens(tokens: &[Token]) -> Vec<String> {
    let mut lines = Vec::new();

    let mut prev = Token::Blank;
    let mut stream = tokens.into_iter();

    loop {
        let curr = if let Some(token) = stream.next() {
            token
        } else {
            break;
        };

        match curr {
            Token::Blank => {
                prev = curr.clone();
            }
            Token::Block(line) => {
                lines.push(line.clone());
                prev = curr.clone();
            }
            Token::Text(line) => {
                if prev == Token::Blank {
                    lines.extend(parse_paragraph(line, &mut stream, &mut prev));
                } else {
                    prev = curr.clone();
                }
            }
            _ => {}
        }
    }

    lines
}

fn parse_paragraph<'a, I>(text: &str, stream: &mut I, prev: &mut Token) -> Vec<String>
where
    I: Iterator<Item = &'a Token>,
{
    let mut lines = vec![format!("<p>{}\n", text)];

    loop {
        let curr = if let Some(token) = stream.next() {
            token
        } else {
            &Token::End
        };

        if let Token::Text(ref text) = curr {
            lines.push(text.clone());
        } else {
            *prev = curr.clone();
            // append the end tag to the last line
            let last_index = lines.len() - 1;
            let last_line = lines.get_mut(last_index).unwrap();
            *last_line = format!("{}</p>\n", last_line.trim_end());
            break;
        }
    }

    lines
}
