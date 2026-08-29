//! Lightweight, High-Performance i18n & Localization Dictionary Engine.
//!
//! Features:
//! - Loads dictionary files (`data/lang/*.toml`) and modular subdirectories (`data/lang/<dict>/*.toml`).
//! - Automatic merge and collision resolution for directory-based dictionaries.
//! - Granular access control policies (`DictAccess::Public`, `DictAccess::Private`, `DictAccess::Shared`).
//! - Guaranteed immutable Public access for system `common` dictionary with override protection.
//! - Multi-level fallback resolution:
//!   - `<dict>.<lang>` -> `<dict>.<fallback>` -> `<dict>.en` -> `common.<lang>` -> `common.<fallback>` -> `common.en` -> `raw_key`.
//! - Hierarchical TOML schema:
//!   - `[config]` (e.g. `fallback = "ru"`, `access = "public" | "private" | { type = "shared", allowed = [...] }`)
//!   - `[templates]` (declarative reusable macro layouts)
//!   - `[vars]` & `[translations.<lang>.vars]` (scoped string variables & constants)
//!   - `[translations.<lang>]` (or legacy flat `[ru]`, `[en]`)
//! - Built-in formatting macros:
//!   - `@{g(text)}` / `@{green(text)}` -> Green text (`\x04`)
//!   - `@{t(text)}` / `@{team(text)}` -> Team color (`\x03`)
//!   - `@{w(text)}` / `@{white(text)}` -> Standard color (`\x01`)
//!   - `@{tag(text)}` -> `^3[\x04text^3]^1`
//! - Programmatic plugin API (`I18n::register_default`, `I18n::set_access`).
//! - Fast macro `tr!` for ergonomic translation formatting.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

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
                    true
                } else if lower == "shared" {
                    allowed.iter().any(|p| p.to_lowercase() == clean_caller)
                } else {
                    false
                }
            }
        }
    }
}

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

        let (compiled_entries, maybe_access) = Compiler::compile(dict_name, &parsed)?;
        let count = compiled_entries.len();

        let clean_dict = dict_name.to_lowercase();

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
        Self::translate_with_caller("", dict_name, lang, key, named_args, pos_args)
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
            Self::format_placeholders(&raw, named_args, pos_args)
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
                return Self::format_placeholders(&raw, named_args, pos_args);
            }

            // 3. Fallback to 'common' dictionary if not found in target
            if dict_key != "common" {
                return Self::lookup_common(&lang_key, key, named_args, pos_args);
            }

            // 4. Final raw key fallback
            Self::format_placeholders(key, named_args, pos_args)
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
        Self::format_placeholders(&raw, named_args, pos_args)
    }

    /// Replaces `{name}`, `{name='default'}`, `{0}`, and `{0='default'}` placeholders in string.
    pub fn format_placeholders(
        template: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        let mut result = String::with_capacity(template.len());
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek()
                    && (next == '{' || next == '}' || next == '$' || next == '@')
                {
                    result.push(next);
                    chars.next();
                    continue;
                }
                result.push('\\');
            } else if c == '{' {
                let mut placeholder = String::new();
                let mut closed = false;
                for p in chars.by_ref() {
                    if p == '}' {
                        closed = true;
                        break;
                    }
                    placeholder.push(p);
                }

                if !closed {
                    result.push('{');
                    result.push_str(&placeholder);
                    continue;
                }

                let (param_name, default_val) = match placeholder.split_once('=') {
                    Some((name, def)) => {
                        let trimmed_def = def.trim().trim_matches('\'').trim_matches('"');
                        (name.trim(), Some(trimmed_def))
                    }
                    None => (placeholder.trim(), None),
                };

                // Check named args
                let mut replaced = false;
                for &(name, value) in named_args {
                    if name == param_name {
                        result.push_str(value);
                        replaced = true;
                        break;
                    }
                }

                // Check positional args if param_name is numeric
                if !replaced
                    && let Ok(idx) = param_name.parse::<usize>()
                    && let Some(&value) = pos_args.get(idx)
                {
                    result.push_str(value);
                    replaced = true;
                }

                // Apply default value if available
                if !replaced {
                    if let Some(def) = default_val {
                        result.push_str(def);
                    } else {
                        // Keep original placeholder if not provided
                        result.push('{');
                        result.push_str(&placeholder);
                        result.push('}');
                    }
                }
            } else {
                result.push(c);
            }
        }

        result
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

/// Compile-time preprocessor for dictionary TOML tables.
struct Compiler<'a> {
    dict_name: &'a str,
    global_vars: HashMap<String, String>,
    global_templates: HashMap<String, String>,
}

impl<'a> Compiler<'a> {
    fn compile(
        dict_name: &'a str,
        table: &toml::Table,
    ) -> Result<(HashMap<DictionaryKey, String>, Option<DictAccess>), String> {
        let mut compiler = Self {
            dict_name,
            global_vars: HashMap::new(),
            global_templates: HashMap::new(),
        };

        let mut parsed_access = None;

        // 1. Parse [config]
        if let Some(toml::Value::Table(cfg)) = table.get("config") {
            if let Some(toml::Value::String(fb)) = cfg.get("fallback") {
                let mut fallbacks = DICT_FALLBACKS.write().unwrap_or_else(|e| e.into_inner());
                fallbacks.insert(dict_name.to_lowercase(), fb.clone());
            }
            if let Some(acc_val) = cfg.get("access")
                && let Ok(access) = acc_val.clone().try_into::<DictAccess>()
            {
                parsed_access = Some(access);
            }
        }

        // 2. Parse [templates]
        if let Some(toml::Value::Table(tmpls)) = table.get("templates") {
            for (name, val) in tmpls {
                if let toml::Value::String(s) = val {
                    compiler.global_templates.insert(name.clone(), s.clone());
                }
            }
        }

        // 3. Parse [vars]
        if let Some(toml::Value::Table(vars)) = table.get("vars") {
            for (name, val) in vars {
                if let toml::Value::String(s) = val {
                    let expanded = compiler.expand_vars_and_builtins(s, &HashMap::new())?;
                    compiler.global_vars.insert(name.clone(), expanded);
                }
            }
        }

        let mut output = HashMap::new();

        // 4. Parse translations: either [translations.<lang>] or flat [ru], [en]
        let (translations_map, lang_vars_map, lang_templates_map) =
            Self::extract_language_tables(table);

        for (lang, entries) in translations_map {
            let mut local_vars = compiler.global_vars.clone();
            if let Some(local_overrides) = lang_vars_map.get(&lang) {
                for (k, v) in local_overrides {
                    let expanded = compiler.expand_vars_and_builtins(v, &local_vars)?;
                    local_vars.insert(k.clone(), expanded);
                }
            }

            let mut local_templates = compiler.global_templates.clone();
            if let Some(local_tmpl_overrides) = lang_templates_map.get(&lang) {
                for (k, v) in local_tmpl_overrides {
                    local_templates.insert(k.clone(), v.clone());
                }
            }

            for (key, raw_val) in entries {
                let fully_expanded =
                    compiler.expand_entry(&raw_val, &local_vars, &local_templates, 0)?;
                output.insert(
                    (dict_name.to_lowercase(), lang.to_lowercase(), key),
                    fully_expanded,
                );
            }
        }

        Ok((output, parsed_access))
    }

    #[allow(clippy::type_complexity)]
    fn extract_language_tables(
        table: &toml::Table,
    ) -> (
        HashMap<String, HashMap<String, String>>,
        HashMap<String, HashMap<String, String>>,
        HashMap<String, HashMap<String, String>>,
    ) {
        let mut trans = HashMap::new();
        let mut l_vars = HashMap::new();
        let mut l_tmpls = HashMap::new();

        if let Some(toml::Value::Table(translations)) = table.get("translations") {
            for (lang_code, val) in translations {
                if let toml::Value::Table(lang_tbl) = val {
                    let mut entries = HashMap::new();
                    for (k, v) in lang_tbl {
                        if k == "vars" {
                            if let toml::Value::Table(vt) = v {
                                let mut vm = HashMap::new();
                                for (vk, vv) in vt {
                                    if let toml::Value::String(s) = vv {
                                        vm.insert(vk.clone(), s.clone());
                                    }
                                }
                                l_vars.insert(lang_code.clone(), vm);
                            }
                        } else if k == "templates" {
                            if let toml::Value::Table(tt) = v {
                                let mut tm = HashMap::new();
                                for (tk, tv) in tt {
                                    if let toml::Value::String(s) = tv {
                                        tm.insert(tk.clone(), s.clone());
                                    }
                                }
                                l_tmpls.insert(lang_code.clone(), tm);
                            }
                        } else if let toml::Value::String(s) = v {
                            entries.insert(k.clone(), s.clone());
                        }
                    }
                    trans.insert(lang_code.clone(), entries);
                }
            }
        } else {
            // Flat mode: [ru], [en], [ru.vars], etc.
            for (section, val) in table {
                if section == "config" || section == "templates" || section == "vars" {
                    continue;
                }
                if let toml::Value::Table(tbl) = val {
                    if let Some((lang, sub)) = section.split_once('.') {
                        if sub == "vars" {
                            let mut vm = HashMap::new();
                            for (vk, vv) in tbl {
                                if let toml::Value::String(s) = vv {
                                    vm.insert(vk.clone(), s.clone());
                                }
                            }
                            l_vars.insert(lang.to_string(), vm);
                        } else if sub == "templates" {
                            let mut tm = HashMap::new();
                            for (tk, tv) in tbl {
                                if let toml::Value::String(s) = tv {
                                    tm.insert(tk.clone(), s.clone());
                                }
                            }
                            l_tmpls.insert(lang.to_string(), tm);
                        }
                    } else {
                        let mut entries = HashMap::new();
                        for (k, v) in tbl {
                            if let toml::Value::String(s) = v {
                                entries.insert(k.clone(), s.clone());
                            }
                        }
                        trans.insert(section.clone(), entries);
                    }
                }
            }
        }

        (trans, l_vars, l_tmpls)
    }

    fn expand_entry(
        &self,
        input: &str,
        vars: &HashMap<String, String>,
        templates: &HashMap<String, String>,
        depth: usize,
    ) -> Result<String, String> {
        if depth > 8 {
            return Err(format!(
                "Infinite recursion / cycle detected in template macros in dict '{}'",
                self.dict_name
            ));
        }

        // 1. Expand variables and built-in color macros
        let mut text = self.expand_vars_and_builtins(input, vars)?;

        // 2. Expand template macros: @{name(args)} or @{templates.name(args)}
        while let Some(call) = Self::find_macro_call(&text) {
            let expanded_macro = self.evaluate_macro_call(&call, vars, templates, depth + 1)?;
            text.replace_range(call.start..call.end, &expanded_macro);
        }

        Ok(text)
    }

    fn expand_vars_and_builtins(
        &self,
        input: &str,
        vars: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek()
                    && (next == '$' || next == '@' || next == '{' || next == '}')
                {
                    result.push('\\');
                    result.push(next);
                    chars.next();
                    continue;
                }
                result.push('\\');
            } else if c == '$' {
                let mut var_name = String::new();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    for ch in chars.by_ref() {
                        if ch == '}' {
                            break;
                        }
                        var_name.push(ch);
                    }
                } else {
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                            var_name.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }

                let clean_name = var_name
                    .strip_prefix("vars.")
                    .unwrap_or(&var_name)
                    .to_string();

                if let Some(val) = vars.get(&clean_name) {
                    result.push_str(val);
                } else {
                    // Unknown var, keep as is
                    result.push('$');
                    result.push_str(&var_name);
                }
            } else {
                result.push(c);
            }
        }

        // Expand built-in color macros: @{g(text)}, @{t(text)}, @{w(text)}, @{tag(text)}
        let mut text = result;
        while let Some(call) = Self::find_macro_call(&text) {
            let macro_name = call.name.to_lowercase();
            if matches!(
                macro_name.as_str(),
                "g" | "green" | "t" | "team" | "w" | "white" | "tag"
            ) {
                let inner_arg = call.pos_args.first().cloned().unwrap_or_default();
                let colored = match macro_name.as_str() {
                    "g" | "green" => format!("\x04{inner_arg}\x01"),
                    "t" | "team" => format!("\x03{inner_arg}\x01"),
                    "w" | "white" => format!("\x01{inner_arg}\x01"),
                    "tag" => format!("^3[\x04{inner_arg}^3]^1"),
                    _ => inner_arg,
                };
                text.replace_range(call.start..call.end, &colored);
            } else {
                break;
            }
        }

        Ok(text)
    }

    fn evaluate_macro_call(
        &self,
        call: &MacroCall,
        vars: &HashMap<String, String>,
        templates: &HashMap<String, String>,
        depth: usize,
    ) -> Result<String, String> {
        let clean_name = call.name.strip_prefix("templates.").unwrap_or(&call.name);

        // Check built-in color macros first
        match clean_name.to_lowercase().as_str() {
            "g" | "green" => {
                let arg = call.pos_args.first().cloned().unwrap_or_default();
                return Ok(format!("\x04{arg}\x01"));
            }
            "t" | "team" => {
                let arg = call.pos_args.first().cloned().unwrap_or_default();
                return Ok(format!("\x03{arg}\x01"));
            }
            "w" | "white" => {
                let arg = call.pos_args.first().cloned().unwrap_or_default();
                return Ok(format!("\x01{arg}\x01"));
            }
            "tag" => {
                let arg = call.pos_args.first().cloned().unwrap_or_default();
                return Ok(format!("^3[\x04{arg}^3]^1"));
            }
            _ => {}
        }

        let template_body = templates.get(clean_name).ok_or_else(|| {
            format!(
                "Template macro '{}' not found in dictionary '{}'",
                clean_name, self.dict_name
            )
        })?;

        let mut formatted = template_body.clone();
        for (k, v) in &call.named_args {
            let pattern = format!("{{{k}}}");
            formatted = formatted.replace(&pattern, v);
        }
        for (i, v) in call.pos_args.iter().enumerate() {
            let pattern = format!("{{{i}}}");
            formatted = formatted.replace(&pattern, v);
        }

        self.expand_entry(&formatted, vars, templates, depth)
    }

    fn find_macro_call(text: &str) -> Option<MacroCall> {
        let mut idx = 0;
        let bytes = text.as_bytes();

        while idx < bytes.len() {
            if bytes[idx] == b'\\' && idx + 1 < bytes.len() && bytes[idx + 1] == b'@' {
                idx += 2;
                continue;
            }

            if bytes[idx] == b'@' && idx + 1 < bytes.len() && bytes[idx + 1] == b'{' {
                let start = idx;
                let mut depth = 1;
                let mut end = start + 2;

                while end < bytes.len() && depth > 0 {
                    if bytes[end] == b'{' {
                        depth += 1;
                    } else if bytes[end] == b'}' {
                        depth -= 1;
                    }
                    end += 1;
                }

                if depth == 0 {
                    let inner = &text[start + 2..end - 1].trim();
                    if let Some((name, args_str)) = inner.split_once('(') {
                        let clean_name = name.trim().to_string();
                        let clean_args = args_str.strip_suffix(')').unwrap_or(args_str).trim();
                        let (pos, named) = Self::parse_macro_args(clean_args);

                        return Some(MacroCall {
                            start,
                            end,
                            name: clean_name,
                            pos_args: pos,
                            named_args: named,
                        });
                    }
                }
            }
            idx += 1;
        }

        None
    }

    fn parse_macro_args(args_str: &str) -> (Vec<String>, Vec<(String, String)>) {
        let mut pos = Vec::new();
        let mut named = Vec::new();

        if args_str.is_empty() {
            return (pos, named);
        }

        let mut current = String::new();
        let mut in_quote = false;
        let mut quote_char = ' ';

        for c in args_str.chars() {
            if !in_quote && (c == '\'' || c == '"') {
                in_quote = true;
                quote_char = c;
            } else if in_quote && c == quote_char {
                in_quote = false;
            } else if !in_quote && c == ',' {
                Self::push_parsed_arg(current.trim(), &mut pos, &mut named);
                current.clear();
                continue;
            }
            current.push(c);
        }

        if !current.trim().is_empty() {
            Self::push_parsed_arg(current.trim(), &mut pos, &mut named);
        }

        (pos, named)
    }

    fn push_parsed_arg(arg: &str, pos: &mut Vec<String>, named: &mut Vec<(String, String)>) {
        if let Some((key, val)) = arg.split_once('=') {
            let clean_k = key.trim().to_string();
            let clean_v = val.trim().trim_matches('\'').trim_matches('"').to_string();
            named.push((clean_k, clean_v));
        } else {
            let clean_v = arg.trim().trim_matches('\'').trim_matches('"').to_string();
            pos.push(clean_v);
        }
    }
}

struct MacroCall {
    start: usize,
    end: usize,
    name: String,
    pos_args: Vec<String>,
    named_args: Vec<(String, String)>,
}

/// Macro for formatted translations using dictionaries.
#[macro_export]
macro_rules! tr {
    ($dict:expr, $lang:expr, $key:expr) => {
        $crate::i18n::I18nEngine::translate($dict, $lang, $key, &[], &[])
    };
    ($dict:expr, $lang:expr, $key:expr, $($name:ident = $val:expr),* $(,)?) => {
        $crate::i18n::I18nEngine::translate(
            $dict,
            $lang,
            $key,
            &[ $((stringify!($name), &$val.to_string())),* ],
            &[],
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_replacement_with_defaults() {
        let template = "Hello, {name='Guest'}! Level: {lvl='1'}, Weapon: {0='Knife'}";
        let named = [("name", "Player1")];
        let pos = [];

        let formatted = I18nEngine::format_placeholders(template, &named, &pos);
        assert_eq!(formatted, "Hello, Player1! Level: 1, Weapon: Knife");
    }

    #[test]
    fn test_advanced_i18n_compilation_and_macros() {
        let toml_content = r#"
            [config]
            fallback = "ru"

            [templates]
            box = "$vars.prefix {0} @{w('(Info: $vars.support_url)')}"
            award = "$vars.prefix Вам выдано: @{g('{item}')}!"

            [vars]
            prefix = "@{tag('VIP System')}"
            support_url = "discord.gg/server"
            currency = "$"

            [translations.ru.vars]
            currency = "₽"

            [translations.ru]
            menu_title = "$vars.prefix Выберите комплект:"
            money_reward = "@{templates.award(item = '{amount} $vars.currency')}"
            info = "@{templates.box('Правила сервера обновлены.')}"

            [translations.en]
            menu_title = "$vars.prefix Choose kit:"
            money_reward = "@{templates.award(item = '{amount} $vars.currency')}"
            info = "@{templates.box('Server rules updated.')}"
        "#;

        I18nEngine::clear();
        let count = I18nEngine::load_toml_string("vip_menu", toml_content).unwrap();
        assert_eq!(count, 6); // 3 ru + 3 en

        // 1. Test Russian currency scoping (₽)
        let ru_reward =
            I18nEngine::translate("vip_menu", "ru", "money_reward", &[("amount", "5000")], &[]);
        assert_eq!(
            ru_reward,
            "^3[\x04VIP System^3]^1 Вам выдано: \x045000 ₽\x01!"
        );

        // 2. Test English currency ($)
        let en_reward =
            I18nEngine::translate("vip_menu", "en", "money_reward", &[("amount", "5000")], &[]);
        assert_eq!(
            en_reward,
            "^3[\x04VIP System^3]^1 Вам выдано: \x045000 $\x01!"
        );

        // 3. Test template macro expansion with positional arg & support url
        let ru_info = I18nEngine::translate("vip_menu", "ru", "info", &[], &[]);
        assert_eq!(
            ru_info,
            "^3[\x04VIP System^3]^1 Правила сервера обновлены. \x01(Info: discord.gg/server)\x01"
        );

        // 4. Test fallback to config.fallback ("ru") when asking for German
        let de_title = I18nEngine::translate("vip_menu", "de", "menu_title", &[], &[]);
        assert_eq!(de_title, "^3[\x04VIP System^3]^1 Выберите комплект:");
    }

    #[test]
    fn test_access_control_and_common_fallback() {
        let common_toml = r#"
            [translations.ru]
            btn_yes = "Да"
            btn_no = "Нет"
            btn_back = "Назад"
        "#;

        let vip_toml = r#"
            [config]
            access = { type = "shared", allowed = ["vip_menu", "vip_chat"] }

            [translations.ru]
            tag = "[VIP Core]"
        "#;

        let secret_toml = r#"
            [config]
            access = "private"

            [translations.ru]
            password = "SecretPassword123"
        "#;

        I18nEngine::clear();
        I18nEngine::load_toml_string("common", common_toml).unwrap();
        I18nEngine::load_toml_string("vip_core", vip_toml).unwrap();
        I18nEngine::load_toml_string("secret_system", secret_toml).unwrap();

        // 1. Owner can access its own private/shared dictionary
        let owner_tag =
            I18nEngine::translate_with_caller("vip_core", "vip_core", "ru", "tag", &[], &[]);
        assert_eq!(owner_tag, "[VIP Core]");

        // 2. Shared plugin can access shared dictionary
        let shared_tag =
            I18nEngine::translate_with_caller("vip_menu", "vip_core", "ru", "tag", &[], &[]);
        assert_eq!(shared_tag, "[VIP Core]");

        // 3. Unauthorized plugin is denied and falls back to common (or raw key)
        let denied = I18nEngine::translate_with_caller(
            "random_plugin",
            "secret_system",
            "ru",
            "password",
            &[],
            &[],
        );
        assert_eq!(denied, "password"); // Key not in common, returns raw key

        // 4. Any plugin can access common phrases even when calling another dict that lacks the key
        let common_btn =
            I18nEngine::translate_with_caller("vip_menu", "vip_core", "ru", "btn_yes", &[], &[]);
        assert_eq!(common_btn, "Да");

        // 5. Common dictionary cannot be locked down to private
        I18nEngine::set_access("common", DictAccess::Simple("private".to_string()), true);
        let common_allowed =
            I18nEngine::translate_with_caller("any_plugin", "common", "ru", "btn_back", &[], &[]);
        assert_eq!(common_allowed, "Назад");
    }

    #[test]
    fn test_directory_and_file_merge() {
        let unique_id = format!(
            "{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let temp_dir = std::env::temp_dir().join(format!("goldsrc_test_lang_{unique_id}"));
        let admin_dir = temp_dir.join("admin_system");
        let _ = std::fs::create_dir_all(&admin_dir);

        let root_file = temp_dir.join("admin_system.toml");
        let root_content = r#"
            [translations.ru]
            title = "Панель управления"
            cmd_kick = "Кикнуть игрока"
        "#;
        std::fs::write(&root_file, root_content).unwrap();

        let sub_file = admin_dir.join("bans.toml");
        let sub_content = r#"
            [translations.ru]
            cmd_ban = "Забанить игрока"
        "#;
        std::fs::write(&sub_file, sub_content).unwrap();

        I18nEngine::clear();
        let count = I18nEngine::load_dir(&temp_dir);
        assert_eq!(count, 3);

        let title = I18nEngine::translate_with_caller(
            "admin_system",
            "admin_system",
            "ru",
            "title",
            &[],
            &[],
        );
        let kick = I18nEngine::translate_with_caller(
            "admin_system",
            "admin_system",
            "ru",
            "cmd_kick",
            &[],
            &[],
        );
        let ban = I18nEngine::translate_with_caller(
            "admin_system",
            "admin_system",
            "ru",
            "cmd_ban",
            &[],
            &[],
        );

        assert_eq!(title, "Панель управления");
        assert_eq!(kick, "Кикнуть игрока");
        assert_eq!(ban, "Забанить игрока");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
