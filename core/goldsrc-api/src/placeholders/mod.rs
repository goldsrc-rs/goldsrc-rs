//! Dynamic Contextual Placeholder Engine abstractions, argument parsers, and metadata definitions.

use crate::client::Player;
use crate::dsl::{Lexer, Token};

/// Represents an argument in a function-like placeholder call: `{name(arg1, key='val')}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArg {
    /// Positional argument value.
    Positional(String),
    /// Named argument key-value pair.
    Named { name: String, value: String },
}

/// Parsed functional placeholder descriptor: `{domain:ident(args...)}` or `{ident='default'}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceholderCall {
    /// Domain/Plugin prefix if specified (e.g. `stats` in `{stats:rank}`).
    pub domain: Option<String>,
    /// Primary placeholder identifier (e.g. `rank`, `ip`, `name`).
    pub ident: String,
    /// Optional default fallback value (e.g. `Guest` in `{name='Guest'}`).
    pub default: Option<String>,
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

    let (prefix_and_ident, default) = match prefix_and_ident.split_once('=') {
        Some((head, def)) => {
            let clean_def = def.trim().trim_matches(['\'', '"']).to_string();
            (head.trim(), Some(clean_def))
        }
        None => (prefix_and_ident, None),
    };

    let (domain, ident) = if let Some((d, id)) = prefix_and_ident.split_once(':') {
        (Some(d.trim().to_string()), id.trim().to_string())
    } else {
        (None, prefix_and_ident.to_string())
    };

    if ident.is_empty() {
        return Err("Placeholder identifier cannot be empty".to_string());
    }

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
        default,
        args,
    })
}

/// Target player resolution strategy for placeholder function calls (e.g. `{ip(target='PlayerName')}`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerTarget {
    /// Resolved by 1-based slot index (1..32).
    Slot(i32),
    /// Resolved by UserID (e.g. `#12`).
    UserId(i32),
    /// Resolved by player display name or substring match.
    Name(String),
    /// Resolved by AuthID / SteamID (e.g. `STEAM_0:0:12345`).
    AuthId(String),
}

impl PlayerTarget {
    /// Parses target argument string into a `PlayerTarget` variant.
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix('#')
            && let Ok(uid) = rest.parse::<i32>()
        {
            return Some(PlayerTarget::UserId(uid));
        }

        if let Ok(slot) = trimmed.parse::<i32>()
            && (1..=32).contains(&slot)
        {
            return Some(PlayerTarget::Slot(slot));
        }

        if trimmed.starts_with("STEAM_")
            || trimmed.starts_with("VALVE_")
            || trimmed.starts_with("BOT")
        {
            return Some(PlayerTarget::AuthId(trimmed.to_string()));
        }

        Some(PlayerTarget::Name(trimmed.to_string()))
    }
}

/// Metadata exported by a plugin for registered placeholders.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct PlaceholderMetadata {
    /// Primary placeholder identifier name (e.g. `rank`, `ip`, `kills`).
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Usage example / parameter signature (e.g. `{rank(format='short')}`).
    #[serde(default)]
    pub usage: String,
    /// List of alternative names or aliases.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Optional required capability for callers to resolve this placeholder.
    #[serde(default)]
    pub capability: Option<String>,
}

/// Trait implemented by native and WASM placeholder providers.
pub trait PlaceholderHandler: Send + Sync {
    /// Evaluates the placeholder function for a given caller and parsed call arguments.
    fn evaluate(&self, caller: Player, call: &PlaceholderCall) -> String;
}

impl<F> PlaceholderHandler for F
where
    F: Fn(Player, &PlaceholderCall) -> String + Send + Sync,
{
    fn evaluate(&self, caller: Player, call: &PlaceholderCall) -> String {
        self(caller, call)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_placeholder_call() {
        let p1 = parse_placeholder_call("name").unwrap();
        assert_eq!(p1.domain, None);
        assert_eq!(p1.ident, "name");
        assert_eq!(p1.default, None);
        assert!(p1.args.is_empty());

        let p2 = parse_placeholder_call("stats:rank(target='bruh', format='short')").unwrap();
        assert_eq!(p2.domain.as_deref(), Some("stats"));
        assert_eq!(p2.ident, "rank");
        assert_eq!(p2.get_named("target"), Some("bruh"));
        assert_eq!(p2.get_named("format"), Some("short"));

        let p3 = parse_placeholder_call("ip('127.0.0.1')").unwrap();
        assert_eq!(p3.ident, "ip");
        assert_eq!(p3.get_positional(0), Some("127.0.0.1"));

        let p4 = parse_placeholder_call("name='Guest'").unwrap();
        assert_eq!(p4.ident, "name");
        assert_eq!(p4.default.as_deref(), Some("Guest"));
    }
}
