// Get HTML lines.
pub fn parse(lines: &[String]) -> Vec<String> {
    let tokens = tokenize(lines);
    parse_tokens(&tokens)
}

enum Token {
    Text(String),
}

// Turn lines of text into block tokens, which'll be turned into MD blocks later.
fn tokenize(lines: &[String]) -> Vec<Token> {
    let mut tokens = Vec::new();

    for line in lines.iter() {
        if !line.is_empty() {
            tokens.push(Token::Text(line.clone()))
        }
    }

    tokens
}

fn parse_tokens(tokens: &[Token]) -> Vec<String> {
    let mut lines = Vec::new();

    for token in tokens {
        let Token::Text(text) = token;
        lines.extend(parse_paragraph(text));
    }

    lines
}

fn parse_paragraph(text: &str) -> Vec<String> {
    text.split("\n")
        .map(|piece| format!("<p>{}</p>", piece))
        .collect::<Vec<String>>()
}
