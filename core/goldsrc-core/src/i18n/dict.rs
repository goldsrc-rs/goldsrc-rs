//! Language dictionary data structures and access models.

use std::collections::HashMap;

/// Access policy for a language dictionary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum DictAccess {
    /// Named variant: "public" or "private".
    Simple(String),
    /// Structured variant with explicit allowlist: `{ type = "shared", allowed = ["plugin1", "plugin2"] }`.
    Structured {
        #[serde(rename = "type")]
        kind: String,
        #[serde(default)]
        allowed: Vec<String>,
    },
}

impl Default for DictAccess {
    fn default() -> Self {
        Self::Simple("private".to_string())
    }
}

impl DictAccess {
    /// Creates a Public access policy.
    pub fn public() -> Self {
        Self::Simple("public".to_string())
    }

    /// Creates a Private access policy.
    pub fn private() -> Self {
        Self::Simple("private".to_string())
    }

    /// Creates a Shared access policy with allowed plugin names.
    pub fn shared(allowed: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Structured {
            kind: "shared".to_string(),
            allowed: allowed.into_iter().map(Into::into).collect(),
        }
    }

    /// Checks if a caller plugin is permitted to access this dictionary.
    pub fn is_allowed(&self, dict_name: &str, caller_plugin: &str) -> bool {
        let clean_dict = dict_name.to_lowercase();
        let clean_caller = caller_plugin.to_lowercase();

        // 1. System 'common' is always accessible to everyone
        if clean_dict == "common" {
            return true;
        }

        // 2. The owner plugin can always access its own dictionary
        if clean_dict == clean_caller || clean_caller.is_empty() {
            return true;
        }

        match self {
            Self::Simple(s) => {
                let lower = s.to_lowercase();
                lower == "public"
            }
            Self::Structured { kind, allowed } => {
                let lower = kind.to_lowercase();
                if lower == "public" {
                    return true;
                }
                if lower == "shared" {
                    return allowed.iter().any(|p| p.to_lowercase() == clean_caller);
                }
                false
            }
        }
    }
}

/// Metadata and behavior settings for a dictionary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DictConfig {
    /// Optional SemVer string of the dictionary schema/contents (e.g. "1.0.0").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Optional author / maintainer of the localization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Default fallback language code (e.g. "en" or "ru").
    #[serde(default = "default_fallback")]
    pub fallback: String,
    /// Access control policy.
    #[serde(default)]
    pub access: DictAccess,
    /// Strict mode: logs error alerts when requested keys or parameters are missing.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub strict_mode: bool,
}

fn default_fallback() -> String {
    "en".to_string()
}

impl Default for DictConfig {
    fn default() -> Self {
        Self {
            version: None,
            author: None,
            fallback: default_fallback(),
            access: DictAccess::default(),
            strict_mode: false,
        }
    }
}

/// A per-language translation table holding scoped vars, scoped templates, and key-value entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LangTable {
    /// Scoped variables that shadow root `[vars]` for this language.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vars: HashMap<String, String>,
    /// Scoped templates for this language.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub templates: HashMap<String, String>,
    /// Key-value translation strings.
    #[serde(flatten)]
    pub entries: HashMap<String, String>,
}

/// Typed language dictionary representation.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LangDict {
    /// Dictionary configuration and metadata.
    #[serde(default, skip_serializing_if = "is_default_config")]
    pub config: DictConfig,
    /// Global macro templates (e.g. `award = "$vars.prefix ..."`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub templates: HashMap<String, String>,
    /// Global variables (e.g. `prefix = "@{tag('VIP')}"`).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub vars: HashMap<String, String>,
    /// Language translation tables.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub translations: HashMap<String, LangTable>,
}

fn is_default_config(cfg: &DictConfig) -> bool {
    cfg == &DictConfig::default()
}

impl LangDict {
    /// Creates a new `LangDictBuilder` for fluent dictionary construction.
    pub fn builder(dict_name: impl Into<String>) -> crate::i18n::builder::LangDictBuilder {
        crate::i18n::builder::LangDictBuilder::new(dict_name)
    }

    /// Serializes this dictionary to a formatted TOML string.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Parses a dictionary from a TOML string.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }
}
