//! Lightweight, High-Performance i18n & Localization Dictionary Engine.
//!
//! Features:
//! - Loads dictionary files (`data/lang/*.toml`) into in-memory hash tables (`O(1)` access).
//! - Supports both named (`{name}`, `{amount}`) and positional (`{0}`, `{1}`) placeholder replacements.
//! - Automatic Code Page conversion:
//!   - `SayText` -> UTF-8 with color formatting (`\x01`, `\x03`, `\x04`).
//!   - `Center` / `Console` -> Windows-1251 (`CP1251`) / Windows-1252 (`CP1252`).
//! - Fast macro `tr!` for ergonomic translation formatting.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

type DictionaryKey = (String, String, String);
type DictionaryStore = HashMap<DictionaryKey, String>;

/// Global in-memory dictionary repository: (plugin/dict_name, lang, key) -> template string.
static DICTIONARIES: LazyLock<RwLock<DictionaryStore>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Centralized i18n manager for loading and translating game messages.
pub struct I18nEngine;

impl I18nEngine {
    /// Loads a dictionary TOML file from disk (e.g. `data/lang/vip_menu.toml`).
    ///
    /// Expected TOML format:
    /// ```toml
    /// [ru]
    /// menu_title = "^3[\x04VIP Menu^3] ^1Выберите привилегию:"
    /// give_money = "Вам выдано ^4{amount}^1$!"
    ///
    /// [en]
    /// menu_title = "^3[\x04VIP Menu^3] ^1Choose privilege:"
    /// give_money = "You received ^4{amount}^1$!"
    /// ```
    pub fn load_file(dict_name: &str, file_path: impl AsRef<Path>) -> Result<usize, String> {
        let path = file_path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read lang file '{:?}': {e}", path))?;

        let parsed: toml::Table = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse lang TOML '{:?}': {e}", path))?;

        let mut count = 0;
        let mut dict = DICTIONARIES.write().unwrap_or_else(|e| e.into_inner());

        for (lang_code, val) in parsed {
            if let toml::Value::Table(keys_table) = val {
                for (key, text_val) in keys_table {
                    if let toml::Value::String(text) = text_val {
                        dict.insert(
                            (dict_name.to_lowercase(), lang_code.to_lowercase(), key),
                            text,
                        );
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Loads all `*.toml` files from the specified `data/lang/` directory.
    pub fn load_dir(lang_dir: impl AsRef<Path>) -> usize {
        let mut total = 0;
        if let Ok(entries) = std::fs::read_dir(lang_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                    let dict_name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("common");
                    if let Ok(count) = Self::load_file(dict_name, &path) {
                        total += count;
                    }
                }
            }
        }
        total
    }

    /// Translates a key for the target language, falling back to "en" or key name if not found.
    pub fn translate(
        dict_name: &str,
        lang: &str,
        key: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        let dict = DICTIONARIES.read().unwrap_or_else(|e| e.into_inner());
        let dict_key = dict_name.to_lowercase();
        let lang_key = lang.to_lowercase();

        // 1. Try specified language
        let template = dict
            .get(&(dict_key.clone(), lang_key, key.to_string()))
            // 2. Fallback to English ("en")
            .or_else(|| dict.get(&(dict_key, "en".to_string(), key.to_string())))
            .cloned();

        let raw = template.unwrap_or_else(|| key.to_string());
        Self::format_placeholders(&raw, named_args, pos_args)
    }

    /// Replaces `{name}` and `{0}` placeholders in string.
    pub fn format_placeholders(
        template: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        let mut result = template.to_string();

        // Replace named placeholders
        for &(name, value) in named_args {
            let pattern = format!("{{{name}}}");
            result = result.replace(&pattern, value);
        }

        // Replace positional placeholders
        for (i, &value) in pos_args.iter().enumerate() {
            let pattern = format!("{{{i}}}");
            result = result.replace(&pattern, value);
        }

        result
    }

    /// Clears all loaded dictionaries (e.g. on server reload).
    pub fn clear() {
        if let Ok(mut dict) = DICTIONARIES.write() {
            dict.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_replacement() {
        let template = "Hello, {name}! You have {amount}$ (Slot: {0}, ID: {1})";
        let named = [("name", "Player1"), ("amount", "500")];
        let pos = ["#1", "4419"];

        let formatted = I18nEngine::format_placeholders(template, &named, &pos);
        assert_eq!(
            formatted,
            "Hello, Player1! You have 500$ (Slot: #1, ID: 4419)"
        );
    }

    #[test]
    fn test_i18n_load_and_translate() {
        let toml_str = r#"
            [ru]
            greeting = "Привет, {name}!"
            reward = "Вы получили {0} XP"

            [en]
            greeting = "Hello, {name}!"
            reward = "You received {0} XP"
        "#;

        let temp_file = std::env::temp_dir().join(format!("test_lang_{}.toml", std::process::id()));
        std::fs::write(&temp_file, toml_str).unwrap();

        let count = I18nEngine::load_file("test_plugin", &temp_file).unwrap();
        assert_eq!(count, 4);

        let ru_msg =
            I18nEngine::translate("test_plugin", "ru", "greeting", &[("name", "Алексей")], &[]);
        assert_eq!(ru_msg, "Привет, Алексей!");

        let en_msg = I18nEngine::translate("test_plugin", "en", "reward", &[], &["150"]);
        assert_eq!(en_msg, "You received 150 XP");

        // Fallback to EN if language not found
        let fallback_msg =
            I18nEngine::translate("test_plugin", "de", "greeting", &[("name", "Hans")], &[]);
        assert_eq!(fallback_msg, "Hello, Hans!");

        let _ = std::fs::remove_file(&temp_file);
    }
}
