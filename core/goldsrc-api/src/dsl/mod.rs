//! Generic Expression DSL tokenizer, AST definitions, and parsing primitives.
//!
//! Provides a unified grammar engine powering Requirements, Capabilities,
//! Placeholders, and Rule condition evaluation across GoldSrc.rs.

/// Token variants recognized by the generic Expression Lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    /// Identifier or symbol (e.g. `stats`, `rank`, `vip_menu`, `admin`).
    Ident(&'a str),
    /// String literal with quotes stripped (e.g. `'de_dust2'`, `"short"`).
    StringLit(&'a str),
    /// Numeric literal integer or float string (e.g. `100`, `1.5`, `-5`).
    NumberLit(&'a str),
    /// Scoped domain separator (`:`).
    Colon,
    /// Namespace property separator (`.`).
    Dot,
    /// Wildcard star (`*`).
    Star,
    /// Comma parameter delimiter (`,`).
    Comma,
    /// Opening parenthesis (`(`).
    OpenParen,
    /// Closing parenthesis (`)`).
    CloseParen,
    /// Opening bracket (`[`).
    OpenBracket,
    /// Closing bracket (`]`).
    CloseBracket,
    /// Opening brace (`{`).
    OpenBrace,
    /// Closing brace (`}`).
    CloseBrace,
    /// Logical AND (`&` or `&&` or `AND`).
    And,
    /// Logical OR (`|` or `||` or `OR`).
    Or,
    /// Logical NOT (`!` or `NOT`).
    Not,
    /// Equal / Assignment (`=` or `==`).
    Eq,
    /// Inequality (`!=`).
    Ne,
    /// Greater than (`>`).
    Gt,
    /// Greater than or equal (`>=`).
    Ge,
    /// Less than (`<`).
    Lt,
    /// Less than or equal (`<=`).
    Le,
    /// Question mark for optional requirement / flag (`?`).
    Question,
    /// At-sign for version requirement (`@`).
    At,
}

/// Zero-allocation lexer for GoldSrc DSL expressions.
pub struct Lexer<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given string slice.
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    /// Tokenizes the entire input into a vector of tokens.
    pub fn tokenize(input: &'a str) -> Result<Vec<Token<'a>>, String> {
        let mut lexer = Self::new(input);
        let mut tokens = Vec::new();
        while let Some(tok) = lexer.next_token()? {
            tokens.push(tok);
        }
        Ok(tokens)
    }

    fn peek_byte(&self) -> Option<u8> {
        if self.pos < self.bytes.len() {
            Some(self.bytes[self.pos])
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek_byte() {
            if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Fetches the next token from input.
    pub fn next_token(&mut self) -> Result<Option<Token<'a>>, String> {
        self.skip_whitespace();
        let b = match self.peek_byte() {
            Some(b) => b,
            None => return Ok(None),
        };

        match b {
            b':' => {
                self.pos += 1;
                Ok(Some(Token::Colon))
            }
            b'.' => {
                self.pos += 1;
                Ok(Some(Token::Dot))
            }
            b'*' => {
                self.pos += 1;
                Ok(Some(Token::Star))
            }
            b',' => {
                self.pos += 1;
                Ok(Some(Token::Comma))
            }
            b'(' => {
                self.pos += 1;
                Ok(Some(Token::OpenParen))
            }
            b')' => {
                self.pos += 1;
                Ok(Some(Token::CloseParen))
            }
            b'[' => {
                self.pos += 1;
                Ok(Some(Token::OpenBracket))
            }
            b']' => {
                self.pos += 1;
                Ok(Some(Token::CloseBracket))
            }
            b'{' => {
                self.pos += 1;
                Ok(Some(Token::OpenBrace))
            }
            b'}' => {
                self.pos += 1;
                Ok(Some(Token::CloseBrace))
            }
            b'?' => {
                self.pos += 1;
                Ok(Some(Token::Question))
            }
            b'@' => {
                self.pos += 1;
                Ok(Some(Token::At))
            }
            b'&' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'&') {
                    self.pos += 1;
                }
                Ok(Some(Token::And))
            }
            b'|' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'|') {
                    self.pos += 1;
                }
                Ok(Some(Token::Or))
            }
            b'!' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok(Some(Token::Ne))
                } else {
                    Ok(Some(Token::Not))
                }
            }
            b'=' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                }
                Ok(Some(Token::Eq))
            }
            b'>' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok(Some(Token::Ge))
                } else {
                    Ok(Some(Token::Gt))
                }
            }
            b'<' => {
                self.pos += 1;
                if self.peek_byte() == Some(b'=') {
                    self.pos += 1;
                    Ok(Some(Token::Le))
                } else {
                    Ok(Some(Token::Lt))
                }
            }
            b'\'' | b'"' => {
                let quote = b;
                self.pos += 1;
                let start = self.pos;
                while self.pos < self.bytes.len() && self.bytes[self.pos] != quote {
                    self.pos += 1;
                }
                if self.pos >= self.bytes.len() {
                    return Err(format!(
                        "Unterminated string literal starting at position {start}"
                    ));
                }
                let s = &self.input[start..self.pos];
                self.pos += 1; // consume closing quote
                Ok(Some(Token::StringLit(s)))
            }
            _ if b.is_ascii_digit()
                || (b == b'-'
                    && self.pos + 1 < self.bytes.len()
                    && self.bytes[self.pos + 1].is_ascii_digit()) =>
            {
                let start = self.pos;
                if b == b'-' {
                    self.pos += 1;
                }
                while self.pos < self.bytes.len()
                    && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'.')
                {
                    self.pos += 1;
                }
                Ok(Some(Token::NumberLit(&self.input[start..self.pos])))
            }
            _ if b.is_ascii_alphanumeric()
                || b == b'_'
                || b == b'/'
                || b == b'-'
                || b == b'~'
                || b == b'^' =>
            {
                let start = self.pos;
                while self.pos < self.bytes.len() {
                    let c = self.bytes[self.pos];
                    if c.is_ascii_alphanumeric()
                        || c == b'_'
                        || c == b'/'
                        || c == b'-'
                        || c == b'~'
                        || c == b'^'
                    {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                let ident = &self.input[start..self.pos];
                match ident {
                    "AND" | "and" => Ok(Some(Token::And)),
                    "OR" | "or" => Ok(Some(Token::Or)),
                    "NOT" | "not" => Ok(Some(Token::Not)),
                    _ => Ok(Some(Token::Ident(ident))),
                }
            }
            _ => {
                let pos = self.pos;
                self.pos += 1;
                Err(format!(
                    "Unexpected character '{}' at byte position {pos}",
                    b as char
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_basic_tokens() {
        let input = "plugin:vip_core@^1.2.0, cvar:sv_restart==0 & !admin.*";
        let tokens = Lexer::tokenize(input).unwrap();
        assert!(tokens.contains(&Token::Colon));
        assert!(tokens.contains(&Token::And));
        assert!(tokens.contains(&Token::Not));
    }
}
