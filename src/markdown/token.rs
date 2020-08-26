#[derive(Debug, PartialEq)]
pub enum RawTokenKind {
    Element,
    Fenced,
}

pub struct RawToken {
    pub kind: RawTokenKind,
    pub tag: String,
    pub is_nest: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Blank,
    EqualsLine,
    DashLine,
    HorizontalRule,
    Head,
    Numbered,
    Bulleted,
    Dt,
    Dd,
    Raw,
    QuoteBlock,
    MarkupBlock,
    Text,
    End,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: String,
    pub indent_level: u32,
}

impl Token {
    pub fn new<T: Into<String>>(kind: TokenKind, line: T, indent_level: u32) -> Self {
        Token {
            kind,
            line: line.into(),
            indent_level,
        }
    }

    #[inline]
    pub fn new_blank() -> Self {
        Token::new(TokenKind::Blank, "", u32::max_value())
    }

    #[inline]
    pub fn new_end() -> Self {
        Token::new(TokenKind::End, "", u32::max_value())
    }

    #[inline]
    pub fn new_raw<T: Into<String>>(line: T) -> Self {
        Token::new(TokenKind::Raw, line, u32::max_value())
    }
}

#[derive(Debug)]
pub struct TokenStream<'a> {
    tokens: &'a [Token],
    curr: usize,
    tab_size: u32,
    // the token before all given tokens
    before: Token,
    // the token after all given tokens
    after: Token,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Token], tab_size: u32) -> Self {
        TokenStream {
            tokens,
            curr: 0,
            tab_size,
            before: Token::new_blank(),
            after: Token::new_end(),
        }
    }

    #[inline]
    pub fn advance(&mut self) {
        if self.curr < self.tokens.len() {
            self.curr += 1;
        }
    }

    #[inline]
    fn nth(&self, index: usize) -> &Token {
        if index >= self.tokens.len() {
            &self.after
        } else {
            &self.tokens[index]
        }
    }

    #[inline]
    pub fn curr(&self) -> &Token {
        self.nth(self.curr)
    }

    #[inline]
    pub fn prev(&self) -> &Token {
        if self.curr == 0 {
            &self.before
        } else {
            &self.nth(self.curr - 1)
        }
    }

    #[inline]
    pub fn next(&self) -> &Token {
        &self.nth(self.curr + 1)
    }

    #[inline]
    pub fn next_next(&self) -> &Token {
        &self.nth(self.curr + 2)
    }

    #[inline]
    pub fn tab_size(&self) -> u32 {
        self.tab_size
    }
}
