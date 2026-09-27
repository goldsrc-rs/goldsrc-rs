//! Hierarchical Capability DSL parser, semantic validator, and AST evaluator.
//!
//! # Grammar:
//! ```text
//! Expr       := Term ( ('|' | 'OR') Term )*
//! Term       := Factor ( ('&' | 'AND' | ',') Factor )*
//! Factor     := ('!' | 'NOT') Factor | '(' Expr ')' | '*' | Group | CapNode
//! Group      := Ident ':' '[' (Expr (',' Expr)*)? ']'
//!             | Ident ':![' (Expr (',' Expr)*)? ']'
//!             | Ident ':*'
//! CapNode    := Ident ( ('.' | ':') Ident | '.*' | ':*' )* ( '(' Args? ')' )?
//! ```

use crate::dsl::{Lexer, Token};
use std::collections::HashSet;

/// Result of capability expression parsing and semantic validation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidationResult {
    /// Parsed capability expression AST.
    pub ast: CapExpr,
    /// Semantic and precedence ambiguity warnings.
    pub warnings: Vec<String>,
}

/// Abstract Syntax Tree (AST) for Capability expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CapExpr {
    /// Exact, hierarchical, or wildcard capability node (e.g. `admin.slay`, `chat:channel(admin)`, `vip.*`).
    Node(String),
    /// Logical NOT (`!expr`).
    Not(Box<CapExpr>),
    /// Logical AND (`expr & expr`).
    And(Vec<CapExpr>),
    /// Logical OR (`expr | expr`).
    Or(Vec<CapExpr>),
}

impl CapExpr {
    /// Parse a DSL expression string into an AST, returning errors on failure.
    pub fn parse(input: &str) -> Result<Self, String> {
        Self::parse_with_diagnostics(input).map(|res| res.ast)
    }

    /// Parse a DSL expression string into an AST with semantic validation and ambiguity diagnostics.
    pub fn parse_with_diagnostics(input: &str) -> Result<ValidationResult, String> {
        let tokens = Lexer::tokenize(input)?;
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_expr()?;
        if !parser.is_eof() {
            return Err(format!("Unexpected trailing token '{:?}'", parser.peek()));
        }
        let warnings = parser.warnings;
        Ok(ValidationResult { ast, warnings })
    }

    /// Renders the AST with explicit parentheses clarifying all operator precedence.
    pub fn to_pretty_string(&self) -> String {
        match self {
            CapExpr::Node(pattern) => pattern.clone(),
            CapExpr::Not(inner) => format!("!{}", inner.to_pretty_string()),
            CapExpr::And(items) => {
                if items.is_empty() {
                    String::new()
                } else if items.len() == 1 {
                    items[0].to_pretty_string()
                } else {
                    let parts: Vec<String> = items.iter().map(|e| e.to_pretty_string()).collect();
                    format!("({})", parts.join(" & "))
                }
            }
            CapExpr::Or(items) => {
                if items.is_empty() {
                    String::new()
                } else if items.len() == 1 {
                    items[0].to_pretty_string()
                } else {
                    let parts: Vec<String> = items.iter().map(|e| e.to_pretty_string()).collect();
                    format!("({})", parts.join(" | "))
                }
            }
        }
    }

    /// Evaluates the expression against a capability checker closure.
    pub fn evaluate<F>(&self, has_cap: &F) -> bool
    where
        F: Fn(&str) -> bool,
    {
        match self {
            CapExpr::Node(pattern) => {
                if let Some(prefix) = pattern
                    .strip_suffix(".*")
                    .or_else(|| pattern.strip_suffix(":*"))
                {
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
            for g in granted {
                if let Some(prefix) = g.strip_suffix(".*").or_else(|| g.strip_suffix(":*"))
                    && let Some(rem) = cap.strip_prefix(prefix)
                    && (rem.starts_with('.') || rem.starts_with(':'))
                {
                    return true;
                }
            }
            false
        })
    }
}

// ---------------------------------------------------------------------------
// Unified Parser & Ambiguity Diagnostic Engine
// ---------------------------------------------------------------------------

struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    pos: usize,
    warnings: Vec<String>,
    in_group: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: Vec<Token<'a>>) -> Self {
        Self {
            tokens,
            pos: 0,
            warnings: Vec::new(),
            in_group: 0,
        }
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn peek_ahead(&self, n: usize) -> Option<&Token<'a>> {
        self.tokens.get(self.pos + n)
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
        let (first_term, mut has_unparenthesized_and) = self.parse_term_annotated()?;
        let mut terms = vec![first_term];
        while let Some(Token::Or) = self.peek() {
            self.next();
            let (next_term, next_is_and) = self.parse_term_annotated()?;
            if next_is_and {
                has_unparenthesized_and = true;
            }
            terms.push(next_term);
        }
        if terms.len() == 1 {
            Ok(terms.remove(0))
        } else {
            let or_expr = CapExpr::Or(terms);
            if has_unparenthesized_and {
                self.warnings.push(format!(
                    "Ambiguous operator precedence in capability expression: consider using parentheses '(' ')' to clarify precedence. Treated as: {}",
                    or_expr.to_pretty_string()
                ));
            }
            Ok(or_expr)
        }
    }

    fn parse_term_annotated(&mut self) -> Result<(CapExpr, bool), String> {
        let first = self.parse_factor()?;
        let mut factors = vec![first];
        let mut has_and = false;
        while let Some(tok) = self.peek() {
            match tok {
                Token::And => {
                    self.next();
                    factors.push(self.parse_factor()?);
                    has_and = true;
                }
                Token::Comma => {
                    if self.in_group > 0
                        || self.peek_ahead(1) == Some(&Token::CloseBracket)
                        || self.peek_ahead(1) == Some(&Token::CloseParen)
                    {
                        break;
                    }
                    self.next();
                    factors.push(self.parse_factor()?);
                    has_and = true;
                }
                _ => break,
            }
        }
        if factors.len() == 1 {
            Ok((factors.remove(0), false))
        } else {
            Ok((CapExpr::And(factors), has_and))
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
            Some(Token::Star) => {
                self.next();
                Ok(CapExpr::Node("*".to_string()))
            }
            Some(Token::Ident(_)) => self.parse_ident_node_or_group(),
            other => Err(format!("Unexpected token in factor: {:?}", other)),
        }
    }

    fn parse_ident_node_or_group(&mut self) -> Result<CapExpr, String> {
        let mut name = match self.next() {
            Some(Token::Ident(s)) => s.to_string(),
            _ => unreachable!(),
        };

        loop {
            match self.peek() {
                Some(Token::Colon) => {
                    match self.peek_ahead(1) {
                        Some(Token::Star) => {
                            self.next(); // consume ':'
                            self.next(); // consume '*'
                            let node_name =
                                if crate::auth::roles::namespaces::is_root_namespace(&name) {
                                    format!("{name}:*")
                                } else {
                                    format!("{name}.*")
                                };
                            return Ok(CapExpr::Node(node_name));
                        }
                        Some(Token::OpenBracket) => {
                            self.next(); // consume ':'
                            self.next(); // consume '['
                            return self.parse_group_inner(&name, false);
                        }
                        Some(Token::Not) if self.peek_ahead(2) == Some(&Token::OpenBracket) => {
                            self.next(); // consume ':'
                            self.next(); // consume '!'
                            self.next(); // consume '['
                            return self.parse_group_inner(&name, true);
                        }
                        Some(Token::Ident(sub)) => {
                            let sub_str = sub.to_string();
                            self.next(); // consume ':'
                            self.next(); // consume ident
                            name.push(':');
                            name.push_str(&sub_str);
                        }
                        other => {
                            return Err(format!(
                                "Expected identifier, '[', or '*' after ':', got {:?}",
                                other
                            ));
                        }
                    }
                }
                Some(Token::Dot) => {
                    self.next(); // consume '.'
                    match self.peek() {
                        Some(Token::Star) => {
                            self.next();
                            name.push_str(".*");
                            return Ok(CapExpr::Node(name));
                        }
                        Some(Token::Ident(sub)) => {
                            let sub_str = sub.to_string();
                            self.next();
                            name.push('.');
                            name.push_str(&sub_str);
                        }
                        other => {
                            return Err(format!(
                                "Expected identifier or '*' after '.', got {:?}",
                                other
                            ));
                        }
                    }
                }
                Some(Token::OpenParen) => {
                    let args = self.parse_call_args()?;
                    name.push_str(&args);
                    return Ok(CapExpr::Node(name));
                }
                _ => break,
            }
        }

        Ok(CapExpr::Node(name))
    }

    fn parse_call_args(&mut self) -> Result<String, String> {
        let mut args_str = String::new();
        self.next(); // consume '('
        let mut first = true;
        while let Some(tok) = self.peek() {
            if *tok == Token::CloseParen {
                break;
            }
            if !first {
                if let Some(Token::Comma) = self.peek() {
                    self.next(); // consume ','
                    args_str.push_str(", ");
                } else {
                    return Err(format!(
                        "Expected ',' between arguments, got {:?}",
                        self.peek()
                    ));
                }
            }
            first = false;

            match self.next() {
                Some(Token::Ident(id)) => {
                    if let Some(Token::Eq) = self.peek() {
                        self.next(); // consume '='
                        match self.next() {
                            Some(Token::Ident(val)) => {
                                args_str.push_str(id);
                                args_str.push('=');
                                args_str.push_str(val);
                            }
                            Some(Token::NumberLit(val)) => {
                                args_str.push_str(id);
                                args_str.push('=');
                                args_str.push_str(val);
                            }
                            Some(Token::StringLit(val)) => {
                                args_str.push_str(id);
                                args_str.push('=');
                                args_str.push_str(val);
                            }
                            other => {
                                return Err(format!(
                                    "Expected argument value after '=', got {:?}",
                                    other
                                ));
                            }
                        }
                    } else {
                        args_str.push_str(id);
                    }
                }
                Some(Token::NumberLit(val)) => {
                    args_str.push_str(val);
                }
                Some(Token::StringLit(val)) => {
                    args_str.push_str(val);
                }
                other => return Err(format!("Unexpected argument token: {:?}", other)),
            }
        }

        match self.next() {
            Some(Token::CloseParen) => Ok(format!("({args_str})")),
            other => Err(format!(
                "Expected ')' closing call arguments, got {:?}",
                other
            )),
        }
    }

    fn parse_group_inner(&mut self, prefix: &str, negated: bool) -> Result<CapExpr, String> {
        if let Some(Token::CloseBracket) = self.peek() {
            return Err("Empty capability group '[]' is forbidden".to_string());
        }

        self.in_group += 1;
        let mut sub_nodes = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::CloseBracket {
                break;
            }
            let sub_expr = self.parse_expr()?;
            let prefixed = prefix_expr(prefix, sub_expr);
            sub_nodes.push(prefixed);

            if let Some(Token::Comma) = self.peek() {
                self.next();
            } else {
                break;
            }
        }
        self.in_group -= 1;

        match self.next() {
            Some(Token::CloseBracket) => {
                if sub_nodes.is_empty() {
                    Err("Empty capability group '[]' is forbidden".to_string())
                } else {
                    let inner = if sub_nodes.len() == 1 {
                        sub_nodes.remove(0)
                    } else {
                        CapExpr::And(sub_nodes)
                    };
                    if negated {
                        Ok(CapExpr::Not(Box::new(inner)))
                    } else {
                        Ok(inner)
                    }
                }
            }
            other => Err(format!("Expected ']' in group, got {:?}", other)),
        }
    }
}

/// Recursively prefixes node names within a group.
fn prefix_expr(prefix: &str, expr: CapExpr) -> CapExpr {
    match expr {
        CapExpr::Node(name) => {
            let full_name = if name == "*" {
                if prefix.ends_with(':') || prefix.ends_with('.') {
                    format!("{prefix}*")
                } else if crate::auth::roles::namespaces::is_root_namespace(prefix) {
                    format!("{prefix}:*")
                } else {
                    format!("{prefix}.*")
                }
            } else if prefix.ends_with(':')
                || prefix.ends_with('.')
                || name.starts_with('.')
                || name.starts_with(':')
            {
                format!("{prefix}{name}")
            } else if crate::auth::roles::namespaces::is_root_namespace(prefix) {
                format!("{prefix}:{name}")
            } else {
                format!("{prefix}.{name}")
            };
            CapExpr::Node(full_name)
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

        // Test root namespace colon wildcard
        let chat_ast = CapExpr::parse("chat:channel(admin)").unwrap();
        caps.clear();
        caps.insert("chat:*".to_string());
        assert!(chat_ast.evaluate_set(&caps));
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

        let engine_ast = CapExpr::parse("engine:*").unwrap();
        assert_eq!(engine_ast, CapExpr::Node("engine:*".to_string()));
    }

    #[test]
    fn test_empty_group_rejected() {
        assert!(CapExpr::parse("admin:[]").is_err());
    }

    #[test]
    fn test_negated_group_syntax() {
        let ast = CapExpr::parse("admin:![slay, kick]").unwrap();
        assert_eq!(
            ast,
            CapExpr::Not(Box::new(CapExpr::And(vec![
                CapExpr::Node("admin.slay".to_string()),
                CapExpr::Node("admin.kick".to_string()),
            ])))
        );
    }

    #[test]
    fn test_parametric_capabilities() {
        let ast = CapExpr::parse("chat:channel(admin) & gameplay:heal(max=150)").unwrap();
        assert_eq!(
            ast,
            CapExpr::And(vec![
                CapExpr::Node("chat:channel(admin)".to_string()),
                CapExpr::Node("gameplay:heal(max=150)".to_string()),
            ])
        );

        // Parametric calls inside groups
        let group_ast = CapExpr::parse("chat:[channel(admin), channel(vip)]").unwrap();
        assert_eq!(
            group_ast,
            CapExpr::And(vec![
                CapExpr::Node("chat:channel(admin)".to_string()),
                CapExpr::Node("chat:channel(vip)".to_string()),
            ])
        );
    }

    #[test]
    fn test_group_boolean_expansion() {
        let ast = CapExpr::parse("vip:[heal | armor]").unwrap();
        assert_eq!(
            ast,
            CapExpr::Or(vec![
                CapExpr::Node("vip.heal".to_string()),
                CapExpr::Node("vip.armor".to_string()),
            ])
        );

        let complex = CapExpr::parse("admin:[slay | kick, ban]").unwrap();
        assert_eq!(
            complex,
            CapExpr::And(vec![
                CapExpr::Or(vec![
                    CapExpr::Node("admin.slay".to_string()),
                    CapExpr::Node("admin.kick".to_string()),
                ]),
                CapExpr::Node("admin.ban".to_string()),
            ])
        );
    }

    #[test]
    fn test_pretty_string() {
        let ast = CapExpr::parse("admin.slay & (vip.heal | vip.armor)").unwrap();
        assert_eq!(
            ast.to_pretty_string(),
            "(admin.slay & (vip.heal | vip.armor))"
        );
    }

    #[test]
    fn test_ambiguity_warning_and_pretty_ast() {
        let res = CapExpr::parse_with_diagnostics("a & b | c").unwrap();
        assert_eq!(res.warnings.len(), 1);
        assert_eq!(
            res.warnings[0],
            "Ambiguous operator precedence in capability expression: consider using parentheses '(' ')' to clarify precedence. Treated as: ((a & b) | c)"
        );

        let res2 = CapExpr::parse_with_diagnostics("a | b & c").unwrap();
        assert_eq!(res2.warnings.len(), 1);
        assert_eq!(
            res2.warnings[0],
            "Ambiguous operator precedence in capability expression: consider using parentheses '(' ')' to clarify precedence. Treated as: (a | (b & c))"
        );

        // Explicit parentheses clarify precedence: 0 warnings
        let res3 = CapExpr::parse_with_diagnostics("(a & b) | c").unwrap();
        assert!(res3.warnings.is_empty());

        let res4 = CapExpr::parse_with_diagnostics("a | (b & c)").unwrap();
        assert!(res4.warnings.is_empty());
    }
}
