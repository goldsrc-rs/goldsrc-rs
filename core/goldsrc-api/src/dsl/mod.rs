//! Universal Generic Expression DSL tokenizer, AST definitions, builder, and parsing primitives.
//!
//! Provides a unified grammar engine powering Requirements, Capabilities,
//! Placeholders, Permissions, and Rule condition evaluation across GoldSrc.rs.

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

/// Represents an argument in a function call node: `fn(arg1, key='val')`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CallArg {
    /// Positional argument.
    Positional(String),
    /// Named argument key-value pair.
    Named { name: String, value: String },
}

/// Canonical AST node representation in the Universal Unified DSL Engine.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExprNode {
    /// Exact, hierarchical, or wildcard resource path.
    ///
    /// E.g. `namespaces: ["plugin", "vip", "auth"]`, `members: ["weapons", "m4a1"]`.
    Path {
        namespaces: Vec<String>,
        members: Vec<String>,
        args: Vec<CallArg>,
    },
    /// Logical NOT (`!expr`).
    Not(Box<ExprNode>),
    /// Logical AND (`expr & expr`).
    And(Vec<ExprNode>),
    /// Logical OR (`expr | expr`).
    Or(Vec<ExprNode>),
}

impl ExprNode {
    /// Parses a DSL string into an `ExprNode` AST.
    pub fn parse(input: &str) -> Result<Self, String> {
        let tokens = Lexer::tokenize(input)?;
        let mut parser = DslParser::new(tokens);
        let expr = parser.parse_expr()?;
        if !parser.is_eof() {
            return Err(format!("Unexpected trailing token '{:?}'", parser.peek()));
        }
        Ok(expr)
    }

    /// Renders the AST back to a canonical DSL string representation.
    pub fn to_dsl_string(&self) -> String {
        match self {
            ExprNode::Path {
                namespaces,
                members,
                args,
            } => {
                let mut out = String::new();
                if !namespaces.is_empty() {
                    out.push_str(&namespaces.join(":"));
                    out.push(':');
                }
                out.push_str(&members.join("."));
                if !args.is_empty() {
                    out.push('(');
                    let arg_strs: Vec<String> = args
                        .iter()
                        .map(|a| match a {
                            CallArg::Positional(val) => format!("'{val}'"),
                            CallArg::Named { name, value } => format!("{name}='{value}'"),
                        })
                        .collect();
                    out.push_str(&arg_strs.join(", "));
                    out.push(')');
                }
                out
            }
            ExprNode::Not(inner) => format!("!{}", inner.to_dsl_string()),
            ExprNode::And(list) => {
                let parts: Vec<String> = list.iter().map(|e| e.to_dsl_string()).collect();
                if parts.len() == 1 {
                    parts[0].clone()
                } else {
                    format!("({})", parts.join(" & "))
                }
            }
            ExprNode::Or(list) => {
                let parts: Vec<String> = list.iter().map(|e| e.to_dsl_string()).collect();
                if parts.len() == 1 {
                    parts[0].clone()
                } else {
                    format!("({})", parts.join(" | "))
                }
            }
        }
    }
}

/// Recursive descent parser for Universal Expression DSL.
struct DslParser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> DslParser<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token<'a>> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn parse_expr(&mut self) -> Result<ExprNode, String> {
        let mut terms = vec![self.parse_term()?];
        while let Some(Token::Or) = self.peek() {
            self.next();
            terms.push(self.parse_term()?);
        }
        if terms.len() == 1 {
            Ok(terms.remove(0))
        } else {
            Ok(ExprNode::Or(terms))
        }
    }

    fn parse_term(&mut self) -> Result<ExprNode, String> {
        let mut factors = vec![self.parse_factor()?];
        while let Some(tok) = self.peek() {
            if *tok == Token::And || *tok == Token::Comma {
                self.next();
                factors.push(self.parse_factor()?);
            } else {
                break;
            }
        }
        if factors.len() == 1 {
            Ok(factors.remove(0))
        } else {
            Ok(ExprNode::And(factors))
        }
    }

    fn parse_factor(&mut self) -> Result<ExprNode, String> {
        match self.peek() {
            Some(Token::Not) => {
                self.next();
                let inner = self.parse_factor()?;
                Ok(ExprNode::Not(Box::new(inner)))
            }
            Some(Token::OpenParen) => {
                self.next();
                let expr = self.parse_expr()?;
                match self.next() {
                    Some(Token::CloseParen) => Ok(expr),
                    other => Err(format!(
                        "Expected ')' after parenthesized expression, got {:?}",
                        other
                    )),
                }
            }
            Some(Token::Ident(_)) | Some(Token::Star) => self.parse_path_or_group(),
            other => Err(format!("Unexpected token in expression: {:?}", other)),
        }
    }

    fn parse_path_or_group(&mut self) -> Result<ExprNode, String> {
        let mut segments: Vec<String> = Vec::new();
        let mut is_colon_chain = true;
        let mut namespaces: Vec<String> = Vec::new();
        let mut members: Vec<String> = Vec::new();

        // 1. Read first identifier or star
        match self.next() {
            Some(Token::Ident(s)) => segments.push(s.to_string()),
            Some(Token::Star) => {
                members.push("*".to_string());
                return Ok(ExprNode::Path {
                    namespaces: Vec::new(),
                    members,
                    args: Vec::new(),
                });
            }
            other => return Err(format!("Expected identifier, got {:?}", other)),
        }

        // 2. Read chained `:` and `.` separators
        loop {
            match self.peek() {
                Some(Token::Colon) => {
                    self.next(); // consume ':'
                    if let Some(Token::OpenBracket) = self.peek() {
                        // It's a scoped group: `prefix:[...]`
                        self.next(); // consume '['
                        namespaces.append(&mut segments);
                        let inner = self.parse_expr()?;
                        match self.next() {
                            Some(Token::CloseBracket) => {
                                return Ok(prefix_expr(&namespaces, inner));
                            }
                            other => {
                                return Err(format!(
                                    "Expected ']' in scoped group, got {:?}",
                                    other
                                ));
                            }
                        }
                    } else if let Some(Token::Star) = self.peek() {
                        self.next();
                        namespaces.append(&mut segments);
                        members.push("*".to_string());
                        break;
                    } else if let Some(Token::Ident(next_id)) = self.peek() {
                        let id_str = next_id.to_string();
                        self.next();
                        namespaces.append(&mut segments);
                        segments.push(id_str);
                        is_colon_chain = true;
                    } else {
                        return Err("Expected identifier, '[', or '*' after ':'".to_string());
                    }
                }
                Some(Token::Dot) => {
                    self.next(); // consume '.'
                    is_colon_chain = false;
                    match self.peek() {
                        Some(Token::Star) => {
                            self.next();
                            segments.push("*".to_string());
                            break;
                        }
                        Some(Token::Ident(next_id)) => {
                            let id_str = next_id.to_string();
                            self.next();
                            segments.push(id_str);
                        }
                        other => {
                            return Err(format!(
                                "Expected identifier or '*' after '.', got {:?}",
                                other
                            ));
                        }
                    }
                }
                _ => break,
            }
        }

        if is_colon_chain {
            if members.is_empty() {
                members.extend(segments);
            }
        } else {
            members.extend(segments);
        }

        // 3. Optional call arguments `(...)`
        let mut args = Vec::new();
        if let Some(Token::OpenParen) = self.peek() {
            self.next(); // consume '('
            while let Some(tok) = self.peek() {
                if *tok == Token::CloseParen {
                    break;
                }
                match self.next() {
                    Some(Token::StringLit(s)) => {
                        args.push(CallArg::Positional(s.to_string()));
                    }
                    Some(Token::NumberLit(n)) => {
                        args.push(CallArg::Positional(n.to_string()));
                    }
                    Some(Token::Ident(name)) => {
                        if let Some(Token::Eq) = self.peek() {
                            self.next(); // consume '='
                            match self.next() {
                                Some(Token::StringLit(val)) => {
                                    args.push(CallArg::Named {
                                        name: name.to_string(),
                                        value: val.to_string(),
                                    });
                                }
                                Some(Token::NumberLit(val)) => {
                                    args.push(CallArg::Named {
                                        name: name.to_string(),
                                        value: val.to_string(),
                                    });
                                }
                                Some(Token::Ident(val)) => {
                                    args.push(CallArg::Named {
                                        name: name.to_string(),
                                        value: val.to_string(),
                                    });
                                }
                                other => {
                                    return Err(format!(
                                        "Expected argument value after '=', got {:?}",
                                        other
                                    ));
                                }
                            }
                        } else {
                            args.push(CallArg::Positional(name.to_string()));
                        }
                    }
                    other => return Err(format!("Unexpected argument token: {:?}", other)),
                }

                if let Some(Token::Comma) = self.peek() {
                    self.next();
                }
            }

            match self.next() {
                Some(Token::CloseParen) => {}
                other => return Err(format!("Expected ')' closing arguments, got {:?}", other)),
            }
        }

        Ok(ExprNode::Path {
            namespaces,
            members,
            args,
        })
    }
}

/// Helper to distribute namespace prefix across an expression in a group.
fn prefix_expr(prefix: &[String], expr: ExprNode) -> ExprNode {
    match expr {
        ExprNode::Path {
            mut namespaces,
            members,
            args,
        } => {
            let mut new_ns = prefix.to_vec();
            new_ns.append(&mut namespaces);
            ExprNode::Path {
                namespaces: new_ns,
                members,
                args,
            }
        }
        ExprNode::Not(inner) => ExprNode::Not(Box::new(prefix_expr(prefix, *inner))),
        ExprNode::And(list) => {
            ExprNode::And(list.into_iter().map(|e| prefix_expr(prefix, e)).collect())
        }
        ExprNode::Or(list) => {
            ExprNode::Or(list.into_iter().map(|e| prefix_expr(prefix, e)).collect())
        }
    }
}

/// Fluent builder for constructing Universal DSL expressions programmatically.
#[derive(Debug, Clone, Default)]
pub struct DslBuilder {
    node: Option<ExprNode>,
}

impl DslBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes a path node.
    pub fn path(namespaces: &[&str], members: &[&str]) -> Self {
        Self {
            node: Some(ExprNode::Path {
                namespaces: namespaces.iter().map(|s| s.to_string()).collect(),
                members: members.iter().map(|s| s.to_string()).collect(),
                args: Vec::new(),
            }),
        }
    }

    /// Appends a logical AND with `other`.
    pub fn and(self, other: DslBuilder) -> Self {
        let node = match (self.node, other.node) {
            (Some(a), Some(b)) => Some(ExprNode::And(vec![a, b])),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        Self { node }
    }

    /// Appends a logical OR with `other`.
    pub fn or(self, other: DslBuilder) -> Self {
        let node = match (self.node, other.node) {
            (Some(a), Some(b)) => Some(ExprNode::Or(vec![a, b])),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        Self { node }
    }

    /// Wraps the current expression in logical NOT.
    pub fn negate(self) -> Self {
        !self
    }

    /// Finalizes the built AST node.
    pub fn build(self) -> Option<ExprNode> {
        self.node
    }

    /// Finalizes and formats into canonical DSL string.
    pub fn to_dsl_string(self) -> String {
        self.node.map(|n| n.to_dsl_string()).unwrap_or_default()
    }
}

impl std::ops::Not for DslBuilder {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            node: self.node.map(|n| ExprNode::Not(Box::new(n))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Not;

    #[test]
    fn test_lexer_basic_tokens() {
        let input = "plugin:vip_core@^1.2.0, cvar:sv_restart==0 & !admin.*";
        let tokens = Lexer::tokenize(input).unwrap();
        assert!(tokens.contains(&Token::Colon));
        assert!(tokens.contains(&Token::And));
        assert!(tokens.contains(&Token::Not));
    }

    #[test]
    fn test_parse_nested_namespaces_and_scoped_group() {
        let input = "plugin:vip:auth:[weapons.m4a1 | weapons.ak47] & !auth:admin.*";
        let ast = ExprNode::parse(input).unwrap();
        let rendered = ast.to_dsl_string();
        assert!(rendered.contains("plugin:vip:auth:weapons.m4a1"));
        assert!(rendered.contains("plugin:vip:auth:weapons.ak47"));
        assert!(rendered.contains("!auth:admin.*"));
    }

    #[test]
    fn test_parse_functional_arguments() {
        let input = "fs:read('configs/*.toml') & cvar:set('sv_gravity', val='800')";
        let ast = ExprNode::parse(input).unwrap();
        if let ExprNode::And(terms) = ast {
            assert_eq!(terms.len(), 2);
            if let ExprNode::Path {
                namespaces,
                members,
                args,
            } = &terms[0]
            {
                assert_eq!(namespaces, &["fs"]);
                assert_eq!(members, &["read"]);
                assert_eq!(args, &[CallArg::Positional("configs/*.toml".to_string())]);
            } else {
                panic!("Expected path node");
            }
        } else {
            panic!("Expected And expression");
        }
    }

    #[test]
    fn test_dsl_builder() {
        let p1 = DslBuilder::path(&["fs"], &["read"]);
        let p2 = DslBuilder::path(&["chat"], &["broadcast"]).not();
        let expr = p1.and(p2);
        assert_eq!(expr.to_dsl_string(), "(fs:read & !chat:broadcast)");
    }
}
