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

/// Represents an argument in a function-like placeholder call: `{name(arg1, key='val')}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArg {
    /// Positional argument value.
    Positional(String),
    /// Named argument key-value pair.
    Named { name: String, value: String },
}

/// Parsed functional placeholder descriptor: `{domain:ident(args...)}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderCall {
    /// Domain/Plugin prefix if specified (e.g. `stats` in `{stats:rank}`).
    pub domain: Option<String>,
    /// Primary placeholder identifier (e.g. `rank`, `ip`, `name`).
    pub ident: String,
    /// List of parsed call arguments.
    pub args: Vec<CallArg>,
}

impl PlaceholderCall {
    /// Returns the positional argument at `index` if present.
    pub fn get_positional(&self, index: usize) -> Option<&str> {
        let mut pos_count = 0;
        for arg in &self.args {
            if let CallArg::Positional(val) = arg {
                if pos_count == index {
                    return Some(val);
                }
                pos_count += 1;
            }
        }
        None
    }

    /// Returns the named argument by key if present.
    pub fn get_named(&self, key: &str) -> Option<&str> {
        for arg in &self.args {
            if let CallArg::Named { name, value } = arg
                && name.eq_ignore_ascii_case(key)
            {
                return Some(value);
            }
        }
        None
    }

    /// Resolves an argument either by name or by positional fallback index.
    pub fn get_param(&self, key: &str, pos_idx: usize) -> Option<&str> {
        self.get_named(key).or_else(|| self.get_positional(pos_idx))
    }
}

/// Parses a placeholder expression inside `{...}` into a structured `PlaceholderCall`.
pub fn parse_placeholder_call(raw: &str) -> Result<PlaceholderCall, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Empty placeholder string".to_string());
    }

    let (prefix_and_ident, args_raw) = if let Some((head, tail)) = trimmed.split_once('(') {
        if !tail.ends_with(')') {
            return Err(format!(
                "Missing closing parenthesis in placeholder '{{{trimmed}}}'"
            ));
        }
        (head.trim(), Some(&tail[..tail.len() - 1]))
    } else {
        (trimmed, None)
    };

    let (domain, ident) = if let Some((d, id)) = prefix_and_ident.split_once(':') {
        (Some(d.trim().to_string()), id.trim().to_string())
    } else {
        (None, prefix_and_ident.to_string())
    };

    let mut args = Vec::new();
    if let Some(raw_args) = args_raw
        && !raw_args.trim().is_empty()
    {
        let tokens = Lexer::tokenize(raw_args)?;
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
                Token::Ident(k) if i + 1 < tokens.len() && tokens[i + 1] == Token::Eq => {
                    let key = (*k).to_string();
                    i += 2; // skip ident and '='
                    if i >= tokens.len() {
                        return Err(format!("Expected value after '=' for argument '{key}'"));
                    }
                    let val = match &tokens[i] {
                        Token::StringLit(s) => (*s).to_string(),
                        Token::Ident(s) | Token::NumberLit(s) => (*s).to_string(),
                        other => {
                            return Err(format!(
                                "Unexpected token '{:?}' for argument '{key}'",
                                other
                            ));
                        }
                    };
                    args.push(CallArg::Named {
                        name: key,
                        value: val,
                    });
                    i += 1;
                }
                Token::StringLit(s) => {
                    args.push(CallArg::Positional((*s).to_string()));
                    i += 1;
                }
                Token::Ident(s) | Token::NumberLit(s) => {
                    args.push(CallArg::Positional((*s).to_string()));
                    i += 1;
                }
                Token::Comma => {
                    i += 1;
                }
                other => {
                    return Err(format!(
                        "Unexpected token in placeholder arguments: '{:?}'",
                        other
                    ));
                }
            }
        }
    }

    Ok(PlaceholderCall {
        domain,
        ident,
        args,
    })
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

    #[test]
    fn test_parse_placeholder_call() {
        let p1 = parse_placeholder_call("name").unwrap();
        assert_eq!(p1.domain, None);
        assert_eq!(p1.ident, "name");
        assert!(p1.args.is_empty());

        let p2 = parse_placeholder_call("stats:rank(target='bruh', format='short')").unwrap();
        assert_eq!(p2.domain.as_deref(), Some("stats"));
        assert_eq!(p2.ident, "rank");
        assert_eq!(p2.get_named("target"), Some("bruh"));
        assert_eq!(p2.get_named("format"), Some("short"));

        let p3 = parse_placeholder_call("ip('127.0.0.1')").unwrap();
        assert_eq!(p3.ident, "ip");
        assert_eq!(p3.get_positional(0), Some("127.0.0.1"));
    }
}
