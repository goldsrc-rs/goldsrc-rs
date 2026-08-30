//! Compile-time template preprocessor and variable expander.

use crate::i18n::dict::DictAccess;
use std::collections::HashMap;

/// Parsed macro call structure: `@{name(arg1, name2='val')}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroCall {
    pub name: String,
    pub pos_args: Vec<String>,
    pub named_args: HashMap<String, String>,
    pub start: usize,
    pub end: usize,
}

/// Flat compiled runtime map for dictionary entries: `(dict, lang, key) -> template`.
pub type CompiledEntries = HashMap<(String, String, String), String>;

/// Result of compiling a dictionary TOML table: `(entries, access, fallback)`.
pub type CompileResult = Result<(CompiledEntries, Option<DictAccess>, Option<String>), String>;

/// Compile-time preprocessor for dictionary TOML tables.
pub struct Compiler<'a> {
    pub dict_name: &'a str,
}

impl<'a> Compiler<'a> {
    pub fn new(dict_name: &'a str) -> Self {
        Self { dict_name }
    }

    /// Compiles a parsed TOML dictionary table into flat runtime key-value maps:
    /// `((dict_name, lang, key), compiled_template)`.
    pub fn compile(dict_name: &'a str, table: &toml::Table) -> CompileResult {
        let compiler = Self::new(dict_name);
        let mut parsed_fallback = None;
        let mut parsed_access = None;

        // 1. Parse [config]
        if let Some(toml::Value::Table(cfg)) = table.get("config") {
            if let Some(toml::Value::String(fb)) = cfg.get("fallback") {
                parsed_fallback = Some(fb.clone());
            }
            if let Some(acc_val) = cfg.get("access")
                && let Ok(access) = acc_val.clone().try_into::<DictAccess>()
            {
                parsed_access = Some(access);
            }
        }

        // 2. Parse [templates]
        let mut global_templates = HashMap::new();
        if let Some(toml::Value::Table(tmpls)) = table.get("templates") {
            for (k, v) in tmpls {
                if let toml::Value::String(s) = v {
                    global_templates.insert(k.clone(), s.clone());
                }
            }
        }

        // 3. Parse [vars]
        let mut global_vars = HashMap::new();
        if let Some(toml::Value::Table(vars)) = table.get("vars") {
            for (k, v) in vars {
                if let toml::Value::String(s) = v {
                    global_vars.insert(k.clone(), s.clone());
                }
            }
        }

        // 4. Parse translations
        let mut trans = HashMap::new();
        let mut l_vars: HashMap<String, HashMap<String, String>> = HashMap::new();
        let mut l_tmpls: HashMap<String, HashMap<String, String>> = HashMap::new();

        if let Some(toml::Value::Table(trans_tbl)) = table.get("translations") {
            for (lang_code, val) in trans_tbl {
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

        // 5. Expand & compile entries
        let mut compiled = HashMap::new();
        for (lang, entries) in trans {
            let mut scoped_vars = global_vars.clone();
            if let Some(lv) = l_vars.get(&lang) {
                for (k, v) in lv {
                    scoped_vars.insert(k.clone(), v.clone());
                }
            }

            let mut scoped_templates = global_templates.clone();
            if let Some(lt) = l_tmpls.get(&lang) {
                for (k, v) in lt {
                    scoped_templates.insert(k.clone(), v.clone());
                }
            }

            // Expand scoped vars themselves
            let mut expanded_vars = HashMap::new();
            for (k, v) in &scoped_vars {
                let exp = compiler.expand_entry(v, &scoped_vars, &scoped_templates, 0)?;
                expanded_vars.insert(k.clone(), exp);
            }

            // Expand scoped templates
            let mut expanded_templates = HashMap::new();
            for (k, v) in &scoped_templates {
                let exp = compiler.expand_entry(v, &expanded_vars, &scoped_templates, 0)?;
                expanded_templates.insert(k.clone(), exp);
            }

            for (key, raw_text) in entries {
                let compiled_str =
                    compiler.expand_entry(&raw_text, &expanded_vars, &expanded_templates, 0)?;
                compiled.insert(
                    (dict_name.to_lowercase(), lang.to_lowercase(), key.clone()),
                    compiled_str,
                );
            }
        }

        Ok((compiled, parsed_access, parsed_fallback))
    }

    pub fn expand_entry(
        &self,
        text: &str,
        vars: &HashMap<String, String>,
        templates: &HashMap<String, String>,
        depth: usize,
    ) -> Result<String, String> {
        if depth > 16 {
            return Err(format!(
                "Recursion limit exceeded in i18n dictionary '{}' while expanding: {}",
                self.dict_name, text
            ));
        }

        let mut current = text.to_string();

        // 1. Substitute variables: $vars.name, $var_name, ${var_name}
        current = self.expand_vars(&current, vars);

        // 2. Expand macros: @{name(args)} or @macro(args)
        while let Some(call) = Self::find_macro_call(&current) {
            let expanded_macro = self.eval_macro(&call, vars, templates, depth + 1)?;
            current.replace_range(call.start..call.end, &expanded_macro);
        }

        Ok(current)
    }

    fn expand_vars(&self, text: &str, vars: &HashMap<String, String>) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' && chars.peek() == Some(&'$') {
                result.push('$');
                chars.next();
                continue;
            }

            if c == '$' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // skip '{'
                    let mut var_name = String::new();
                    let mut closed = false;
                    for vc in chars.by_ref() {
                        if vc == '}' {
                            closed = true;
                            break;
                        }
                        var_name.push(vc);
                    }
                    if closed {
                        let clean_name = var_name.strip_prefix("vars.").unwrap_or(&var_name);
                        if let Some(val) = vars.get(clean_name) {
                            result.push_str(val);
                        } else {
                            result.push_str("${");
                            result.push_str(&var_name);
                            result.push('}');
                        }
                    } else {
                        result.push_str("${");
                        result.push_str(&var_name);
                    }
                    continue;
                }

                // Check for $vars.name or $name
                let mut var_ident = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' || nc == '.' {
                        var_ident.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if var_ident.is_empty() {
                    result.push('$');
                    continue;
                }

                let clean_ident = var_ident.strip_prefix("vars.").unwrap_or(&var_ident);
                if let Some(val) = vars.get(clean_ident) {
                    result.push_str(val);
                } else if let Some(val) = vars.get(&var_ident) {
                    result.push_str(val);
                } else {
                    result.push('$');
                    result.push_str(&var_ident);
                }
            } else {
                result.push(c);
            }
        }

        result
    }

    fn eval_macro(
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

    pub fn find_macro_call(text: &str) -> Option<MacroCall> {
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i].1 == '\\' && i + 1 < chars.len() && chars[i + 1].1 == '@' {
                i += 2;
                continue;
            }

            if chars[i].1 == '@' && i + 1 < chars.len() && chars[i + 1].1 == '{' {
                let start = chars[i].0;
                i += 2;

                let mut name = String::new();
                while i < chars.len()
                    && (chars[i].1.is_alphanumeric() || chars[i].1 == '_' || chars[i].1 == '.')
                {
                    name.push(chars[i].1);
                    i += 1;
                }

                if i < chars.len() && chars[i].1 == '(' {
                    i += 1; // skip '('
                    let mut paren_depth = 1;
                    let mut args_raw = String::new();
                    let mut in_quote: Option<char> = None;

                    while i < chars.len() && paren_depth > 0 {
                        let c = chars[i].1;
                        if let Some(q) = in_quote {
                            if c == q && (i == 0 || chars[i - 1].1 != '\\') {
                                in_quote = None;
                            }
                            args_raw.push(c);
                        } else if c == '\'' || c == '"' {
                            in_quote = Some(c);
                            args_raw.push(c);
                        } else if c == '(' {
                            paren_depth += 1;
                            args_raw.push(c);
                        } else if c == ')' {
                            paren_depth -= 1;
                            if paren_depth > 0 {
                                args_raw.push(c);
                            }
                        } else {
                            args_raw.push(c);
                        }
                        i += 1;
                    }

                    if i < chars.len() && chars[i].1 == '}' {
                        let end = chars[i].0 + chars[i].1.len_utf8();

                        let (pos_args, named_args) = Self::parse_args(&args_raw);
                        return Some(MacroCall {
                            name,
                            pos_args,
                            named_args,
                            start,
                            end,
                        });
                    }
                }
            }
            i += 1;
        }

        None
    }

    fn parse_args(raw: &str) -> (Vec<String>, HashMap<String, String>) {
        let mut pos = Vec::new();
        let mut named = HashMap::new();

        let mut tokens = Vec::new();
        let mut cur = String::new();
        let mut in_quote: Option<char> = None;

        for c in raw.chars() {
            if let Some(q) = in_quote {
                if c == q {
                    in_quote = None;
                }
                cur.push(c);
            } else if c == '\'' || c == '"' {
                in_quote = Some(c);
                cur.push(c);
            } else if c == ',' {
                tokens.push(cur.trim().to_string());
                cur.clear();
            } else {
                cur.push(c);
            }
        }
        if !cur.trim().is_empty() {
            tokens.push(cur.trim().to_string());
        }

        for token in tokens {
            if let Some((k, v)) = token.split_once('=') {
                let clean_v = v.trim().trim_matches('\'').trim_matches('"');
                named.insert(k.trim().to_string(), clean_v.to_string());
            } else {
                let clean_v = token.trim().trim_matches('\'').trim_matches('"');
                pos.push(clean_v.to_string());
            }
        }

        (pos, named)
    }
}
