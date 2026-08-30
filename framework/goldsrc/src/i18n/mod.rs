//! Centralized i18n subsystem for GoldSrc.rs.
//!
//! Provides compile-time template macro expansion, lexical variable scoping,
//! access control policies, directory-based modular loading, and the fluent `LangDictBuilder` API.

pub mod builder;
pub mod compiler;
pub mod dict;
pub mod placeholders;

#[cfg(test)]
mod tests;

pub use builder::{DictConfigBuilder, LangDictBuilder, LangTableBuilder};
pub use compiler::{Compiler, MacroCall};
pub use dict::{DictAccess, DictConfig, LangDict, LangTable};
pub use placeholders::format_placeholders;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

type DictionaryKey = (String, String, String);
type DictionaryStore = HashMap<DictionaryKey, String>;
type FallbackStore = HashMap<String, String>;
type AccessStore = HashMap<String, DictAccess>;

/// Global in-memory dictionary repository: (plugin/dict_name, lang, key) -> template string.
static DICTIONARIES: LazyLock<RwLock<DictionaryStore>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Dictionary-specific fallback languages: dict_name -> fallback_lang.
static DICT_FALLBACKS: LazyLock<RwLock<FallbackStore>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Dictionary-specific access control policies: dict_name -> DictAccess.
static DICT_ACCESS: LazyLock<RwLock<AccessStore>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Centralized i18n manager for loading and translating game messages.
pub struct I18nEngine;

impl I18nEngine {
    /// Loads a dictionary TOML file from disk (e.g. `data/lang/vip_menu.toml`).
    pub fn load_file(dict_name: &str, file_path: impl AsRef<Path>) -> Result<usize, String> {
        let path = file_path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read lang file '{:?}': {e}", path))?;

        Self::load_toml_string(dict_name, &content)
    }

    /// Parses and registers a TOML string into the dictionary store.
    pub fn load_toml_string(dict_name: &str, toml_str: &str) -> Result<usize, String> {
        let parsed: toml::Table = toml::from_str(toml_str)
            .map_err(|e| format!("Failed to parse lang TOML for '{dict_name}': {e}"))?;

        let (compiled_entries, maybe_access, maybe_fallback) =
            Compiler::compile(dict_name, &parsed)?;
        let count = compiled_entries.len();

        let clean_dict = dict_name.to_lowercase();

        // Register fallback language if defined
        if let Some(fb) = maybe_fallback {
            let mut fallbacks = DICT_FALLBACKS.write().unwrap_or_else(|e| e.into_inner());
            fallbacks.insert(clean_dict.clone(), fb);
        }

        // Register access policy if defined
        if let Some(access) = maybe_access {
            Self::set_access(&clean_dict, access, true);
        } else if clean_dict == "common" {
            Self::set_access("common", DictAccess::Simple("public".to_string()), true);
        }

        let mut dict = DICTIONARIES.write().unwrap_or_else(|e| e.into_inner());
        for ((d, l, k), val) in compiled_entries {
            dict.insert((d, l, k), val);
        }

        Ok(count)
    }

    /// Sets access policy for a dictionary.
    pub fn set_access(dict_name: &str, access: DictAccess, from_disk_config: bool) {
        let clean_dict = dict_name.to_lowercase();

        // Guarantee immutable Public status for 'common'
        if clean_dict == "common" {
            let is_public = match &access {
                DictAccess::Simple(s) => s.to_lowercase() == "public",
                DictAccess::Structured { kind, .. } => kind.to_lowercase() == "public",
            };
            if !is_public {
                log::warn!(
                    target: "i18n",
                    "Dictionary 'common' is system-level and cannot be set to non-public access. Ignored, remains 'Public'"
                );
            }
            if let Ok(mut lock) = DICT_ACCESS.write() {
                lock.insert(
                    "common".to_string(),
                    DictAccess::Simple("public".to_string()),
                );
            }
            return;
        }

        if let Ok(mut lock) = DICT_ACCESS.write() {
            if from_disk_config && lock.contains_key(&clean_dict) {
                log::info!(
                    target: "i18n",
                    "Dictionary '{clean_dict}' access policy updated by disk config: {:?}",
                    access
                );
            }
            lock.insert(clean_dict, access);
        }
    }

    /// Loads all `*.toml` files and modular directories from the specified `data/lang/` directory.
    pub fn load_dir(lang_dir: impl AsRef<Path>) -> usize {
        let dir = lang_dir.as_ref();
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
        }

        let mut total = 0;
        let mut single_files = HashMap::new();
        let mut subdirs = HashMap::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        single_files.insert(stem.to_string(), path);
                    }
                } else if path.is_dir()
                    && let Some(dir_name) = path.file_name().and_then(|s| s.to_str())
                {
                    subdirs.insert(dir_name.to_string(), path);
                }
            }
        }

        // 1. Process single files
        for (dict_name, file_path) in &single_files {
            if subdirs.contains_key(dict_name) {
                log::debug!(
                    target: "i18n",
                    "Found both file '{dict_name}.toml' and directory '{dict_name}/'. Merging into dictionary '{dict_name}'."
                );
            }
            if let Ok(count) = Self::load_file(dict_name, file_path) {
                total += count;
            }
        }

        // 2. Process directories (recursive merging)
        for (dict_name, dir_path) in &subdirs {
            if let Ok(sub_entries) = std::fs::read_dir(dir_path) {
                for sub_entry in sub_entries.flatten() {
                    let sub_path = sub_entry.path();
                    if sub_path.is_file()
                        && sub_path.extension().is_some_and(|ext| ext == "toml")
                        && let Ok(count) = Self::load_file(dict_name, &sub_path)
                    {
                        total += count;
                    }
                }
            }
        }

        total
    }

    /// Translates a key with caller-based access check, target fallback, and common fallback.
    pub fn translate(
        dict_name: &str,
        lang: &str,
        key: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        Self::translate_with_caller(dict_name, dict_name, lang, key, named_args, pos_args)
    }

    /// Translates a key with explicit caller plugin verification.
    pub fn translate_with_caller(
        caller_plugin: &str,
        dict_name: &str,
        lang: &str,
        key: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = caller_plugin;
            let raw =
                goldsrc_api::bindings::goldsrc::engine::api::host_translate(dict_name, lang, key);
            format_placeholders(&raw, named_args, pos_args)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dict_key = dict_name.to_lowercase();
            let lang_key = lang.to_lowercase();

            // 1. Access Control Check
            let is_allowed = {
                let access_lock = DICT_ACCESS.read().unwrap_or_else(|e| e.into_inner());
                let access = access_lock.get(&dict_key).cloned().unwrap_or_default();
                access.is_allowed(&dict_key, caller_plugin)
            };

            if !is_allowed {
                log::warn!(
                    target: "i18n",
                    "Access denied: plugin '{caller_plugin}' attempted to access private dictionary '{dict_name}'"
                );
                // Fallback to 'common' dictionary
                return Self::lookup_common(&lang_key, key, named_args, pos_args);
            }

            // 2. Lookup in Target Dictionary
            let fallback_lang = {
                let fallbacks = DICT_FALLBACKS.read().unwrap_or_else(|e| e.into_inner());
                fallbacks
                    .get(&dict_key)
                    .cloned()
                    .unwrap_or_else(|| "en".to_string())
            };

            let dict = DICTIONARIES.read().unwrap_or_else(|e| e.into_inner());

            // Priority: target.lang -> target.dict_fallback -> target.en
            let template = dict
                .get(&(dict_key.clone(), lang_key.clone(), key.to_string()))
                .or_else(|| {
                    dict.get(&(
                        dict_key.clone(),
                        fallback_lang.to_lowercase(),
                        key.to_string(),
                    ))
                })
                .or_else(|| dict.get(&(dict_key.clone(), "en".to_string(), key.to_string())))
                .cloned();

            drop(dict);

            if let Some(raw) = template {
                return format_placeholders(&raw, named_args, pos_args);
            }

            // 3. Fallback to 'common' dictionary if not found in target
            if dict_key != "common" {
                return Self::lookup_common(&lang_key, key, named_args, pos_args);
            }

            // 4. Final raw key fallback
            format_placeholders(key, named_args, pos_args)
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn lookup_common(
        lang_key: &str,
        key: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        let dict = DICTIONARIES.read().unwrap_or_else(|e| e.into_inner());
        let common_key = "common".to_string();

        let common_fallback = {
            let fallbacks = DICT_FALLBACKS.read().unwrap_or_else(|e| e.into_inner());
            fallbacks
                .get(&common_key)
                .cloned()
                .unwrap_or_else(|| "en".to_string())
        };

        let template = dict
            .get(&(common_key.clone(), lang_key.to_string(), key.to_string()))
            .or_else(|| {
                dict.get(&(
                    common_key.clone(),
                    common_fallback.to_lowercase(),
                    key.to_string(),
                ))
            })
            .or_else(|| dict.get(&(common_key, "en".to_string(), key.to_string())))
            .cloned();

        let raw = template.unwrap_or_else(|| key.to_string());
        format_placeholders(&raw, named_args, pos_args)
    }

    /// Returns the global server default language code (from `server_language` cvar or fallback "en").
    pub fn server_lang() -> String {
        goldsrc_api::engine_api::cvar_get_string("server_language")
            .unwrap_or_else(|| "en".to_string())
    }

    /// Clears all loaded dictionaries.
    pub fn clear() {
        if let Ok(mut dict) = DICTIONARIES.write() {
            dict.clear();
        }
        if let Ok(mut fallbacks) = DICT_FALLBACKS.write() {
            fallbacks.clear();
        }
        if let Ok(mut access) = DICT_ACCESS.write() {
            access.clear();
        }
    }
}

/// Macro for translating keys from dictionaries with optional named & positional arguments.
/// Accepts language codes (`&str`, `String`) as well as `&Player` or `Player`.
///
/// # Examples
/// ```ignore
/// let msg = tr!("vip_menu", &player, "welcome", name = "Player");
/// let msg = tr!("vip_menu", "ru", "award", "1000", "Knife");
/// ```
#[macro_export]
macro_rules! tr {
    ($dict:expr, $lang:expr, $key:expr) => {{
        use $crate::AsLangCode as _;
        $crate::i18n::I18nEngine::translate($dict, (&$lang).as_lang_code().as_ref(), $key, &[], &[])
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        use $crate::AsLangCode as _;
        let named: &[(&str, &str)] = &[
            $( (stringify!($k), &$v.to_string()) ),*
        ];
        $crate::i18n::I18nEngine::translate($dict, (&$lang).as_lang_code().as_ref(), $key, named, &[])
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $pos:expr ),* $(,)?) => {{
        use $crate::AsLangCode as _;
        let pos: &[&str] = &[
            $( &$pos.to_string() ),*
        ];
        $crate::i18n::I18nEngine::translate($dict, (&$lang).as_lang_code().as_ref(), $key, &[], pos)
    }};
}
