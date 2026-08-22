//! Hierarchical Capability DSL parser and AST evaluator.
//!
//! # Grammar:
//! ```text
//! Expr       := Term ( ('|' | 'OR') Term )*
//! Term       := Factor ( ('&' | 'AND' | ',') Factor )*
//! Factor     := ('!' | 'NOT') Factor | '(' Expr ')' | Group | CapNode
//! Group      := Ident ':' '[' (Expr (',' Expr)*)? ']'
//!             | Ident ':![' (Expr (',' Expr)*)? ']'
//!             | Ident ':*'
//! CapNode    := Ident ( '.' Ident | '.*' )*
//! ```

use std::collections::HashSet;

/// Abstract Syntax Tree (AST) for Capability expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapExpr {
    /// Exact or wildcard capability node (e.g. `admin.slay`, `vip.*`).
    Node(String),
    /// Logical NOT (`!expr`).
    Not(Box<CapExpr>),
    /// Logical AND (`expr & expr`).
    And(Vec<CapExpr>),
    /// Logical OR (`expr | expr`).
    Or(Vec<CapExpr>),
}

impl CapExpr {
    /// Parse a DSL expression string into an AST.
    pub fn parse(input: &str) -> Result<Self, String> {
        let tokens = Lexer::tokenize(input)?;
        let mut parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        if !parser.is_eof() {
            return Err(format!("Unexpected trailing token '{:?}'", parser.peek()));
        }
        Ok(expr)
    }

    /// Evaluates the expression against a capability checker closure.
    pub fn evaluate<F>(&self, has_cap: &F) -> bool
    where
        F: Fn(&str) -> bool,
    {
        match self {
            CapExpr::Node(pattern) => {
                if let Some(prefix) = pattern.strip_suffix(".*") {
                    has_cap(pattern) || has_cap(prefix) || has_cap("*")
                } else {
                    has_cap(pattern) || has_cap("*")
                }
            }
            CapExpr::Not(inner) => !inner.evaluate(has_cap),
            CapExpr::And(items) => items.iter().all(|item| item.evaluate(has_cap)),
            CapExpr::Or(items) => items.iter().any(|item| item.evaluate(has_cap)),
        }
    }

    /// Evaluates against a set of granted capability strings (with wildcard resolution).
    pub fn evaluate_set(&self, granted: &HashSet<String>) -> bool {
        self.evaluate(&|cap| {
            if granted.contains(cap) || granted.contains("*") {
                return true;
            }
            // Check wildcard matches: e.g. granted "admin.*" matches required "admin.slay"
            for g in granted {
                if let Some(prefix) = g.strip_suffix(".*")
                    && cap.starts_with(prefix)
                    && cap[prefix.len()..].starts_with('.')
                {
                    return true;
                }
            }
            false
        })
    }
}

// ---------------------------------------------------------------------------
// Lexer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Ident(String),
    And,
    Or,
    Not,
    Colon,
    Comma,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Star,
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn tokenize(input: &'a str) -> Result<Vec<Token>, String> {
        let mut lexer = Self {
            chars: input.chars().peekable(),
        };
        let mut tokens = Vec::new();

        while let Some(&ch) = lexer.chars.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    lexer.chars.next();
                }
                '&' => {
                    lexer.chars.next();
                    tokens.push(Token::And);
                }
                '|' => {
                    lexer.chars.next();
                    tokens.push(Token::Or);
                }
                '!' => {
                    lexer.chars.next();
                    tokens.push(Token::Not);
                }
                ':' => {
                    lexer.chars.next();
                    tokens.push(Token::Colon);
                }
                ',' => {
                    lexer.chars.next();
                    tokens.push(Token::Comma);
                }
                '(' => {
                    lexer.chars.next();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    lexer.chars.next();
                    tokens.push(Token::RParen);
                }
                '[' => {
                    lexer.chars.next();
                    tokens.push(Token::LBracket);
                }
                ']' => {
                    lexer.chars.next();
                    tokens.push(Token::RBracket);
                }
                '*' => {
                    lexer.chars.next();
                    tokens.push(Token::Star);
                }
                _ if ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == '-' => {
                    let mut s = String::new();
                    while let Some(&c) = lexer.chars.peek() {
                        if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '*' {
                            s.push(c);
                            lexer.chars.next();
                        } else {
                            break;
                        }
                    }
                    match s.to_uppercase().as_str() {
                        "AND" => tokens.push(Token::And),
                        "OR" => tokens.push(Token::Or),
                        "NOT" => tokens.push(Token::Not),
                        _ => tokens.push(Token::Ident(s)),
                    }
                }
                _ => return Err(format!("Unexpected character in DSL: '{ch}'")),
            }
        }
        Ok(tokens)
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn parse_expr(&mut self) -> Result<CapExpr, String> {
        let mut terms = vec![self.parse_term()?];
        while let Some(Token::Or) = self.peek() {
            self.next();
            terms.push(self.parse_term()?);
        }
        if terms.len() == 1 {
            Ok(terms.remove(0))
        } else {
            Ok(CapExpr::Or(terms))
        }
    }

    fn parse_term(&mut self) -> Result<CapExpr, String> {
        let mut factors = vec![self.parse_factor()?];
        while let Some(tok) = self.peek() {
            match tok {
                Token::And | Token::Comma => {
                    self.next();
                    factors.push(self.parse_factor()?);
                }
                _ => break,
            }
        }
        if factors.len() == 1 {
            Ok(factors.remove(0))
        } else {
            Ok(CapExpr::And(factors))
        }
    }

    fn parse_factor(&mut self) -> Result<CapExpr, String> {
        match self.peek() {
            Some(Token::Not) => {
                self.next();
                let inner = self.parse_factor()?;
                Ok(CapExpr::Not(Box::new(inner)))
            }
            Some(Token::LParen) => {
                self.next();
                let expr = self.parse_expr()?;
                match self.next() {
                    Some(Token::RParen) => Ok(expr),
                    other => Err(format!("Expected ')' after expression, got {:?}", other)),
                }
            }
            Some(Token::Ident(_)) => {
                let name = match self.next() {
                    Some(Token::Ident(s)) => s,
                    _ => unreachable!(),
                };

                // Check for group syntax: `prefix:[...]` or `prefix:*`
                if let Some(Token::Colon) = self.peek() {
                    self.next(); // consume ':'
                    match self.peek() {
                        Some(Token::Star) => {
                            self.next();
                            Ok(CapExpr::Node(format!("{name}.*")))
                        }
                        Some(Token::LBracket) => {
                            self.next();
                            let mut sub_nodes = Vec::new();
                            while let Some(tok) = self.peek() {
                                if *tok == Token::RBracket {
                                    break;
                                }
                                let sub_expr = self.parse_factor()?;
                                let prefixed = prefix_expr(&name, sub_expr);
                                sub_nodes.push(prefixed);

                                if let Some(Token::Comma) = self.peek() {
                                    self.next();
                                }
                            }
                            match self.next() {
                                Some(Token::RBracket) => {
                                    if sub_nodes.len() == 1 {
                                        Ok(sub_nodes.remove(0))
                                    } else {
                                        Ok(CapExpr::And(sub_nodes))
                                    }
                                }
                                other => Err(format!("Expected ']' in group, got {:?}", other)),
                            }
                        }
                        other => Err(format!("Expected '[' or '*' after ':', got {:?}", other)),
                    }
                } else {
                    Ok(CapExpr::Node(name))
                }
            }
            other => Err(format!("Unexpected token in factor: {:?}", other)),
        }
    }
}

/// Recursively prefixes node names within a group.
fn prefix_expr(prefix: &str, expr: CapExpr) -> CapExpr {
    match expr {
        CapExpr::Node(name) => {
            if name.starts_with('.') {
                CapExpr::Node(format!("{prefix}{name}"))
            } else {
                CapExpr::Node(format!("{prefix}.{name}"))
            }
        }
        CapExpr::Not(inner) => CapExpr::Not(Box::new(prefix_expr(prefix, *inner))),
        CapExpr::And(list) => {
            CapExpr::And(list.into_iter().map(|e| prefix_expr(prefix, e)).collect())
        }
        CapExpr::Or(list) => {
            CapExpr::Or(list.into_iter().map(|e| prefix_expr(prefix, e)).collect())
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_node() {
        let ast = CapExpr::parse("admin.slay").unwrap();
        assert_eq!(ast, CapExpr::Node("admin.slay".to_string()));

        let mut caps = HashSet::new();
        caps.insert("admin.slay".to_string());
        assert!(ast.evaluate_set(&caps));

        caps.clear();
        caps.insert("admin.kick".to_string());
        assert!(!ast.evaluate_set(&caps));
    }

    #[test]
    fn test_and_or_not_logic() {
        let ast = CapExpr::parse("admin.slay & (vip.heal | vip.armor) & !banned").unwrap();

        let mut caps = HashSet::new();
        caps.insert("admin.slay".to_string());
        caps.insert("vip.heal".to_string());
        assert!(ast.evaluate_set(&caps));

        caps.insert("banned".to_string());
        assert!(!ast.evaluate_set(&caps));
    }

    #[test]
    fn test_wildcard_evaluation() {
        let ast = CapExpr::parse("admin.teleport").unwrap();

        let mut caps = HashSet::new();
        caps.insert("admin.*".to_string());
        assert!(ast.evaluate_set(&caps));

        caps.clear();
        caps.insert("*".to_string());
        assert!(ast.evaluate_set(&caps));
    }

    #[test]
    fn test_group_syntax() {
        let ast = CapExpr::parse("admin:[slay, teleport, !rcon]").unwrap();
        assert_eq!(
            ast,
            CapExpr::And(vec![
                CapExpr::Node("admin.slay".to_string()),
                CapExpr::Node("admin.teleport".to_string()),
                CapExpr::Not(Box::new(CapExpr::Node("admin.rcon".to_string())))
            ])
        );

        let mut caps = HashSet::new();
        caps.insert("admin.slay".to_string());
        caps.insert("admin.teleport".to_string());
        assert!(ast.evaluate_set(&caps));

        caps.insert("admin.rcon".to_string());
        assert!(!ast.evaluate_set(&caps));
    }

    #[test]
    fn test_group_wildcard_syntax() {
        let ast = CapExpr::parse("admin:*").unwrap();
        assert_eq!(ast, CapExpr::Node("admin.*".to_string()));
    }
}
