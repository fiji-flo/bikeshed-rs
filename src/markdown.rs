// Get HTML lines.
pub fn parse(lines: &[String]) -> Vec<String> {
    let tokens = tokenize(lines);
    parse_tokens(&tokens)
}

#[derive(PartialEq, Clone)]
enum Token {
    Blank,
    Text(String),
}

// Turn lines of text into block tokens, which'll be turned into MD blocks later.
fn tokenize(lines: &[String]) -> Vec<Token> {
    let mut tokens = Vec::new();

    for line in lines.iter() {
        if line.is_empty() {
            tokens.push(Token::Blank);
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

        if let Token::Text(text) = curr {
            if prev == Token::Blank {
                lines.extend(parse_paragraph(text, &mut stream, &mut prev));
            }
        } else {
            prev = curr.clone();
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
            break;
        };

        if let Token::Text(ref text) = curr {
            lines.push(text.clone());
        } else {
            *prev = curr.clone();
            let last_index = lines.len() - 1;
            let last_line = lines.get_mut(last_index).unwrap();
            *last_line += "</p>\n";
            break;
        }
    }

    lines
}
