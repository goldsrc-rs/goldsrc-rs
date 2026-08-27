//! Unified Requirements DSL for GoldSrc.rs plugins, systems, and commands.
//!
//! Provides a canonical syntax for specifying runtime and compile-time requirements:
//! - `plugin:<name>[@<semver>]` or `plugin:<name>?`
//! - `cvar:<name>[=<val> | !=<val> | >0 | !=0]`
//! - `feature:<name>`
//! - `engine:<name>` or `host:<semver>`

use std::str::FromStr;

/// Comparison operator for CVar requirement expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CvarOp {
    /// Exact string / numeric equality (`cvar:name=1`).
    Equal(String),
    /// Inequality (`cvar:name!=0`).
    NotEqual(String),
    /// Numeric greater than zero (`cvar:name>0`).
    GreaterThanZero,
}

/// Parsed requirement entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Requirement {
    /// Dependency on another WASM plugin.
    Plugin {
        name: String,
        version_req: Option<String>,
        optional: bool,
    },
    /// Requirement for a specific server or plugin CVar value.
    Cvar { name: String, op: CvarOp },
    /// Host or engine feature capability flag.
    Feature { name: String },
    /// Host runtime version requirement.
    HostVersion { version_req: String },
    /// Engine type requirement (e.g. "goldsrc", "cs16").
    Engine { name: String },
}

impl FromStr for Requirement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err("Empty requirement string".to_string());
        }

        if let Some(rest) = trimmed.strip_prefix("plugin:") {
            let optional = rest.ends_with('?');
            let clean = if optional {
                &rest[..rest.len() - 1]
            } else {
                rest
            };
            if let Some((name, ver)) = clean.split_once('@') {
                return Ok(Requirement::Plugin {
                    name: name.trim().to_string(),
                    version_req: Some(ver.trim().to_string()),
                    optional,
                });
            } else {
                return Ok(Requirement::Plugin {
                    name: clean.trim().to_string(),
                    version_req: None,
                    optional,
                });
            }
        }

        if let Some(rest) = trimmed.strip_prefix("cvar:") {
            if let Some((name, val)) = rest.split_once("!=") {
                return Ok(Requirement::Cvar {
                    name: name.trim().to_string(),
                    op: CvarOp::NotEqual(val.trim().to_string()),
                });
            }
            if let Some((name, val)) = rest.split_once('=') {
                return Ok(Requirement::Cvar {
                    name: name.trim().to_string(),
                    op: CvarOp::Equal(val.trim().to_string()),
                });
            }
            if let Some((name, _)) = rest.split_once(">0") {
                return Ok(Requirement::Cvar {
                    name: name.trim().to_string(),
                    op: CvarOp::GreaterThanZero,
                });
            }
            // Default "cvar:foo" means not equal to 0
            return Ok(Requirement::Cvar {
                name: rest.trim().to_string(),
                op: CvarOp::NotEqual("0".to_string()),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("feature:") {
            return Ok(Requirement::Feature {
                name: rest.trim().to_string(),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("host:") {
            return Ok(Requirement::HostVersion {
                version_req: rest.trim().to_string(),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("engine:") {
            return Ok(Requirement::Engine {
                name: rest.trim().to_string(),
            });
        }

        // Implicit plugin dependency if no prefix is provided
        let optional = trimmed.ends_with('?');
        let clean = if optional {
            &trimmed[..trimmed.len() - 1]
        } else {
            trimmed
        };
        if let Some((name, ver)) = clean.split_once('@') {
            Ok(Requirement::Plugin {
                name: name.trim().to_string(),
                version_req: Some(ver.trim().to_string()),
                optional,
            })
        } else {
            Ok(Requirement::Plugin {
                name: clean.trim().to_string(),
                version_req: None,
                optional,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirements() {
        assert_eq!(
            Requirement::from_str("plugin:vip_core@>=0.10.0").unwrap(),
            Requirement::Plugin {
                name: "vip_core".to_string(),
                version_req: Some(">=0.10.0".to_string()),
                optional: false
            }
        );

        assert_eq!(
            Requirement::from_str("plugin:vip_core?").unwrap(),
            Requirement::Plugin {
                name: "vip_core".to_string(),
                version_req: None,
                optional: true
            }
        );

        assert_eq!(
            Requirement::from_str("cvar:vip_enabled=1").unwrap(),
            Requirement::Cvar {
                name: "vip_enabled".to_string(),
                op: CvarOp::Equal("1".to_string())
            }
        );

        assert_eq!(
            Requirement::from_str("cvar:mp_friendlyfire!=0").unwrap(),
            Requirement::Cvar {
                name: "mp_friendlyfire".to_string(),
                op: CvarOp::NotEqual("0".to_string())
            }
        );

        assert_eq!(
            Requirement::from_str("feature:screen_fade").unwrap(),
            Requirement::Feature {
                name: "screen_fade".to_string()
            }
        );
    }
}
