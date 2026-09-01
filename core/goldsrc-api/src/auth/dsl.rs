//! Hierarchical Capability DSL parser and AST evaluator unified on `goldsrc_api::dsl`.
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

use crate::dsl::{Lexer, Token};
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
    /// Parse a DSL expression string into an AST using the unified DSL Lexer.
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
// Unified Parser
// ---------------------------------------------------------------------------

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
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
            Some(Token::OpenParen) => {
                self.next();
                let expr = self.parse_expr()?;
                match self.next() {
                    Some(Token::CloseParen) => Ok(expr),
                    other => Err(format!("Expected ')' after expression, got {:?}", other)),
                }
            }
            Some(Token::Ident(_)) => {
                let mut name = match self.next() {
                    Some(Token::Ident(s)) => s.to_string(),
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
                        Some(Token::OpenBracket) => {
                            self.next();
                            let mut sub_nodes = Vec::new();
                            while let Some(tok) = self.peek() {
                                if *tok == Token::CloseBracket {
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
                                Some(Token::CloseBracket) => {
                                    if sub_nodes.is_empty() {
                                        Err("Empty capability group '[]' is forbidden".to_string())
                                    } else if sub_nodes.len() == 1 {
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
                    // Consume subsequent `.ident` or `.*` segments
                    while let Some(Token::Dot) = self.peek() {
                        self.next(); // consume '.'
                        match self.peek() {
                            Some(Token::Star) => {
                                self.next();
                                name.push_str(".*");
                                break;
                            }
                            Some(Token::Ident(sub)) => {
                                name.push('.');
                                name.push_str(sub);
                                self.next();
                            }
                            other => {
                                return Err(format!(
                                    "Expected identifier or '*' after '.', got {:?}",
                                    other
                                ));
                            }
                        }
                    }
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

    #[test]
    fn test_empty_group_rejected() {
        assert!(CapExpr::parse("admin:[]").is_err());
    }
}
