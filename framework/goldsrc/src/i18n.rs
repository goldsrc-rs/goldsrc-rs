//! Lightweight, High-Performance i18n & Localization Dictionary Engine.
//!
//! Features:
//! - Loads dictionary files (`data/lang/*.toml`) into in-memory hash tables (`O(1)` access).
//! - Hierarchical TOML schema:
//!   - `[config]` (e.g. `fallback = "ru"`)
//!   - `[templates]` (declarative reusable macro layouts)
//!   - `[vars]` & `[translations.<lang>.vars]` (scoped string variables & constants)
//!   - `[translations.<lang>]` (or legacy flat `[ru]`, `[en]`)
//! - Built-in formatting macros:
//!   - `@{g(text)}` / `@{green(text)}` -> Green text (`\x04`)
//!   - `@{t(text)}` / `@{team(text)}` -> Team color (`\x03`)
//!   - `@{w(text)}` / `@{white(text)}` -> Standard color (`\x01`)
//!   - `@{tag(text)}` -> `^3[\x04text^3]^1`
//! - Macro calls: `@{templates.macro_name(param1, key='val')}`
//! - Variables: `$vars.name`, `$name`, `${name}`
//! - Runtime placeholders with default values: `{name='Guest'}`, `{0='Unknown'}`
//! - Automatic Code Page & escape sequence handling.
//! - Fast macro `tr!` for ergonomic translation formatting.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

type DictionaryKey = (String, String, String);
type DictionaryStore = HashMap<DictionaryKey, String>;
type FallbackStore = HashMap<String, String>;

/// Global in-memory dictionary repository: (plugin/dict_name, lang, key) -> template string.
static DICTIONARIES: LazyLock<RwLock<DictionaryStore>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Dictionary-specific fallback languages: dict_name -> fallback_lang.
static DICT_FALLBACKS: LazyLock<RwLock<FallbackStore>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

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

        let compiled_entries = Compiler::compile(dict_name, &parsed)?;
        let count = compiled_entries.len();

        let mut dict = DICTIONARIES.write().unwrap_or_else(|e| e.into_inner());
        for ((d, l, k), val) in compiled_entries {
            dict.insert((d, l, k), val);
        }

        Ok(count)
    }

    /// Loads all `*.toml` files from the specified `data/lang/` directory.
    pub fn load_dir(lang_dir: impl AsRef<Path>) -> usize {
        let dir = lang_dir.as_ref();
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
        }

        let mut total = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
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

    /// Translates a key for the target language, falling back to dict fallback, then "en", then key name.
    pub fn translate(
        dict_name: &str,
        lang: &str,
        key: &str,
        named_args: &[(&str, &str)],
        pos_args: &[&str],
    ) -> String {
        #[cfg(target_arch = "wasm32")]
        {
            let raw =
                goldsrc_api::bindings::goldsrc::engine::api::host_translate(dict_name, lang, key);
            Self::format_placeholders(&raw, named_args, pos_args)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dict_key = dict_name.to_lowercase();
            let lang_key = lang.to_lowercase();

            let fallback_lang = {
                let fallbacks = DICT_FALLBACKS.read().unwrap_or_else(|e| e.into_inner());
                fallbacks
                    .get(&dict_key)
                    .cloned()
                    .unwrap_or_else(|| "en".to_string())
            };

            let dict = DICTIONARIES.read().unwrap_or_else(|e| e.into_inner());

            // 1. Try specified language
            let template = dict
                .get(&(dict_key.clone(), lang_key, key.to_string()))
                // 2. Try dictionary-specific fallback language
                .or_else(|| {
                    dict.get(&(
                        dict_key.clone(),
                        fallback_lang.to_lowercase(),
                        key.to_string(),
                    ))
                })
                // 3. Fallback to English ("en")
                .or_else(|| dict.get(&(dict_key, "en".to_string(), key.to_string())))
                .cloned();

            let raw = template.unwrap_or_else(|| key.to_string());
            Self::format_placeholders(&raw, named_args, pos_args)
        }
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
    ) -> Result<HashMap<DictionaryKey, String>, String> {
        let mut compiler = Self {
            dict_name,
            global_vars: HashMap::new(),
            global_templates: HashMap::new(),
        };

        // 1. Parse [config]
        if let Some(toml::Value::Table(cfg)) = table.get("config")
            && let Some(toml::Value::String(fb)) = cfg.get("fallback")
        {
            let mut fallbacks = DICT_FALLBACKS.write().unwrap_or_else(|e| e.into_inner());
            fallbacks.insert(dict_name.to_lowercase(), fb.clone());
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

        Ok(output)
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
}
