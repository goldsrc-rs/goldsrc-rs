//! Fluent builder API for constructing language dictionaries.

use crate::i18n::dict::{DictAccess, DictConfig, LangDict, LangTable};
use std::collections::HashMap;

/// Builder for constructing configuration of a language dictionary.
#[derive(Debug, Default, Clone)]
pub struct DictConfigBuilder {
    config: DictConfig,
}

impl DictConfigBuilder {
    /// Sets the dictionary SemVer version string (e.g. "1.0.0").
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = Some(version.into());
        self
    }

    /// Sets the dictionary author or maintainer name.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.config.author = Some(author.into());
        self
    }

    /// Sets the fallback language code for this dictionary (e.g. "en" or "ru").
    pub fn fallback(mut self, lang: impl Into<String>) -> Self {
        self.config.fallback = lang.into();
        self
    }

    /// Sets the access control policy.
    pub fn access(mut self, access: DictAccess) -> Self {
        self.config.access = access;
        self
    }

    /// Enables or disables strict validation mode.
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }

    /// Builds the `DictConfig`.
    pub fn build(self) -> DictConfig {
        self.config
    }
}

/// Builder for constructing per-language translation tables.
#[derive(Debug, Default, Clone)]
pub struct LangTableBuilder {
    table: LangTable,
}

impl LangTableBuilder {
    /// Adds a language-scoped variable (shadows root `[vars]`).
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.table.vars.insert(key.into(), value.into());
        self
    }

    /// Adds a language-scoped template macro.
    pub fn template(mut self, name: impl Into<String>, body: impl Into<String>) -> Self {
        self.table.templates.insert(name.into(), body.into());
        self
    }

    /// Adds a key-value translation entry.
    pub fn entry(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.table.entries.insert(key.into(), value.into());
        self
    }

    /// Extends entries from an iterator of `(key, value)` pairs.
    pub fn entries<K, V>(mut self, iter: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        for (k, v) in iter {
            self.table.entries.insert(k.into(), v.into());
        }
        self
    }

    /// Builds the `LangTable`.
    pub fn build(self) -> LangTable {
        self.table
    }
}

/// Fluent builder for constructing `LangDict` dictionaries.
#[derive(Debug, Clone)]
pub struct LangDictBuilder {
    dict_name: String,
    config: DictConfig,
    templates: HashMap<String, String>,
    vars: HashMap<String, String>,
    translations: HashMap<String, LangTable>,
}

impl LangDictBuilder {
    /// Starts building a new dictionary with the specified name.
    pub fn new(dict_name: impl Into<String>) -> Self {
        Self {
            dict_name: dict_name.into(),
            config: DictConfig::default(),
            templates: HashMap::new(),
            vars: HashMap::new(),
            translations: HashMap::new(),
        }
    }

    /// Configures dictionary settings using a configuration closure.
    pub fn config(
        mut self,
        configure: impl FnOnce(DictConfigBuilder) -> DictConfigBuilder,
    ) -> Self {
        let builder = DictConfigBuilder {
            config: self.config,
        };
        self.config = configure(builder).build();
        self
    }

    /// Top-level shortcut: Sets the default fallback language code.
    pub fn fallback(mut self, lang: impl Into<String>) -> Self {
        self.config.fallback = lang.into();
        self
    }

    /// Top-level shortcut: Sets the access control policy.
    pub fn access(mut self, access: DictAccess) -> Self {
        self.config.access = access;
        self
    }

    /// Top-level shortcut: Sets the dictionary version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.version = Some(version.into());
        self
    }

    /// Top-level shortcut: Sets the dictionary author.
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.config.author = Some(author.into());
        self
    }

    /// Top-level shortcut: Sets strict validation mode.
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }

    /// Adds a global template macro (e.g. `award = "$vars.prefix ..."`).
    pub fn template(mut self, name: impl Into<String>, body: impl Into<String>) -> Self {
        self.templates.insert(name.into(), body.into());
        self
    }

    /// Adds a global variable (e.g. `prefix = "@{tag('VIP')}"`).
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Adds or updates a language translation table via a builder closure.
    pub fn lang(
        mut self,
        lang_code: impl Into<String>,
        configure: impl FnOnce(LangTableBuilder) -> LangTableBuilder,
    ) -> Self {
        let code = lang_code.into().to_lowercase();
        let existing = self.translations.remove(&code).unwrap_or_default();
        let builder = LangTableBuilder { table: existing };
        let built = configure(builder).build();
        self.translations.insert(code, built);
        self
    }

    /// Returns the target dictionary name.
    pub fn dict_name(&self) -> &str {
        &self.dict_name
    }

    /// Builds the `LangDict`.
    pub fn build(self) -> LangDict {
        LangDict {
            config: self.config,
            templates: self.templates,
            vars: self.vars,
            translations: self.translations,
        }
    }
}
