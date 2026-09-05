//! Centralized i18n subsystem for GoldSrc.rs.
//!
//! Provides compile-time template macro expansion, lexical variable scoping,
//! access control policies, directory-based modular loading, and the fluent `LangDictBuilder` API.

pub mod builder;
pub mod compiler;
pub mod dict;
pub mod placeholders;

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

/// Centralized i18n service for loading and translating game messages.
pub struct I18nService;

impl I18nService {
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
        use goldsrc_api::AsLangCode as _;
        $crate::i18n::I18nService::translate($dict, (&$lang).as_lang_code().as_ref(), $key, &[], &[])
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $k:ident = $v:expr ),* $(,)?) => {{
        use goldsrc_api::AsLangCode as _;
        let __owned_vals = [ $( $v.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __named: &[(&str, &str)] = &[
            $( (stringify!($k), __owned_iter.next().unwrap().as_str()) ),*
        ];
        $crate::i18n::I18nService::translate($dict, (&$lang).as_lang_code().as_ref(), $key, __named, &[])
    }};
    ($dict:expr, $lang:expr, $key:expr, $( $pos:expr ),* $(,)?) => {{
        use goldsrc_api::AsLangCode as _;
        let __owned_vals = [ $( $pos.to_string() ),* ];
        let mut __owned_iter = __owned_vals.iter();
        let __pos: &[&str] = &[
            $( __owned_iter.next().unwrap().as_str() ),*
        ];
        $crate::i18n::I18nService::translate($dict, (&$lang).as_lang_code().as_ref(), $key, &[], __pos)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Global test lock to serialize tests that mutate global `I18nService` state.
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_lang_dict_builder_and_toml_serialization() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let dict = LangDict::builder("vip_menu")
            .config(|c| {
                c.version("1.2.0")
                    .author("GoldSrc Team")
                    .fallback("ru")
                    .access(DictAccess::shared(["vip_chat", "vip_core"]))
                    .strict_mode(true)
            })
            .template("award", "$vars.prefix Вам выдано: @{g('{item}')}!")
            .var("prefix", "@{tag('VIP System')}")
            .var("currency", "$")
            .lang("ru", |l| {
                l.var("currency", "₽")
                    .entry("title", "$vars.prefix Выберите комплект:")
                    .entry(
                        "money_reward",
                        "@{templates.award(item = '{amount} $vars.currency')}",
                    )
            })
            .lang("en", |l| {
                l.entry("title", "$vars.prefix Choose kit:").entry(
                    "money_reward",
                    "@{templates.award(item = '{amount} $vars.currency')}",
                )
            })
            .build();

        assert_eq!(dict.config.version.as_deref(), Some("1.2.0"));
        assert_eq!(dict.config.author.as_deref(), Some("GoldSrc Team"));
        assert_eq!(dict.config.fallback, "ru");
        assert!(dict.config.strict_mode);

        // Test serialization to TOML
        let toml_str = dict
            .to_toml()
            .expect("Serialization to TOML should succeed");
        assert!(toml_str.contains("version = \"1.2.0\""));
        assert!(toml_str.contains("author = \"GoldSrc Team\""));
        assert!(toml_str.contains("fallback = \"ru\""));
        assert!(toml_str.contains("strict_mode = true"));

        // Test roundtrip deserialization
        let parsed: LangDict =
            LangDict::from_toml(&toml_str).expect("Parsing from TOML should succeed");
        assert_eq!(parsed, dict);

        // Test loading into I18nService
        I18nService::clear();
        let count = I18nService::load_toml_string("vip_menu", &toml_str).unwrap();
        assert_eq!(count, 4);

        let ru_reward =
            I18nService::translate("vip_menu", "ru", "money_reward", &[("amount", "5000")], &[]);
        assert_eq!(
            ru_reward,
            "^3[\x04VIP System^3]^1 Вам выдано: \x045000 ₽\x01!"
        );

        let en_reward =
            I18nService::translate("vip_menu", "en", "money_reward", &[("amount", "5000")], &[]);
        assert_eq!(
            en_reward,
            "^3[\x04VIP System^3]^1 Вам выдано: \x045000 $\x01!"
        );
    }

    #[test]
    fn test_placeholder_replacement_with_defaults() {
        let template = "Hello, {name='Guest'}! Level: {lvl='1'}, Weapon: {0='Knife'}";
        let named = [("name", "Player1")];
        let pos = [];

        let formatted = format_placeholders(template, &named, &pos);
        assert_eq!(formatted, "Hello, Player1! Level: 1, Weapon: Knife");
    }

    #[test]
    fn test_advanced_i18n_compilation_and_macros() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
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

        I18nService::clear();
        let count = I18nService::load_toml_string("vip_menu", toml_content).unwrap();
        assert_eq!(count, 6); // 3 ru + 3 en

        // 1. Test Russian currency scoping (₽)
        let ru_reward =
            I18nService::translate("vip_menu", "ru", "money_reward", &[("amount", "5000")], &[]);
        assert_eq!(
            ru_reward,
            "^3[\x04VIP System^3]^1 Вам выдано: \x045000 ₽\x01!"
        );

        // 2. Test English currency ($)
        let en_reward =
            I18nService::translate("vip_menu", "en", "money_reward", &[("amount", "5000")], &[]);
        assert_eq!(
            en_reward,
            "^3[\x04VIP System^3]^1 Вам выдано: \x045000 $\x01!"
        );

        // 3. Test template macro expansion with positional arg & support url
        let ru_info = I18nService::translate("vip_menu", "ru", "info", &[], &[]);
        assert_eq!(
            ru_info,
            "^3[\x04VIP System^3]^1 Правила сервера обновлены. \x01(Info: discord.gg/server)\x01"
        );

        // 4. Test fallback to config.fallback ("ru") when asking for German
        let de_title = I18nService::translate("vip_menu", "de", "menu_title", &[], &[]);
        assert_eq!(de_title, "^3[\x04VIP System^3]^1 Выберите комплект:");
    }

    #[test]
    fn test_access_control_and_common_fallback() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
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

        I18nService::clear();
        I18nService::load_toml_string("common", common_toml).unwrap();
        I18nService::load_toml_string("vip_core", vip_toml).unwrap();
        I18nService::load_toml_string("secret_system", secret_toml).unwrap();

        // 1. Owner can access its own private/shared dictionary
        let owner_tag =
            I18nService::translate_with_caller("vip_core", "vip_core", "ru", "tag", &[], &[]);
        assert_eq!(owner_tag, "[VIP Core]");

        // 2. Shared plugin can access shared dictionary
        let shared_tag =
            I18nService::translate_with_caller("vip_menu", "vip_core", "ru", "tag", &[], &[]);
        assert_eq!(shared_tag, "[VIP Core]");

        // 3. Unauthorized plugin is denied and falls back to common (or raw key)
        let denied = I18nService::translate_with_caller(
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
            I18nService::translate_with_caller("vip_menu", "vip_core", "ru", "btn_yes", &[], &[]);
        assert_eq!(common_btn, "Да");

        // 5. Common dictionary cannot be locked down to private
        I18nService::set_access("common", DictAccess::Simple("private".to_string()), true);
        let common_allowed =
            I18nService::translate_with_caller("any_plugin", "common", "ru", "btn_back", &[], &[]);
        assert_eq!(common_allowed, "Назад");
    }

    #[test]
    fn test_directory_and_file_merge() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
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

        I18nService::clear();
        let count = I18nService::load_dir(&temp_dir);
        assert_eq!(count, 3);

        let title = I18nService::translate_with_caller(
            "admin_system",
            "admin_system",
            "ru",
            "title",
            &[],
            &[],
        );
        let kick = I18nService::translate_with_caller(
            "admin_system",
            "admin_system",
            "ru",
            "cmd_kick",
            &[],
            &[],
        );
        let ban = I18nService::translate_with_caller(
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

    #[test]
    fn test_macro_with_nested_placeholder_defaults_and_player_lang() {
        let _lock = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        let toml_content = r#"
            [vars]
            tag = "@{tag('GoldSrc.rs')}"

            [translations.ru]
            welcome_msg = "$vars.tag Добро пожаловать, @{g('{name='Гость'}')}!"
            reward_msg = "$vars.tag Бонус: @{g('{amount='1000'}')} ₽ и @{g('{xp='50'}')} XP!"

            [translations.en]
            welcome_msg = "$vars.tag Welcome, @{g('{name='Guest'}')}!"
            reward_msg = "$vars.tag Bonus: @{g('{amount='1000'}')} $ and @{g('{xp='50'}')} XP!"
        "#;

        I18nService::clear();
        let count = I18nService::load_toml_string("demo_i18n", toml_content).unwrap();
        assert_eq!(count, 4);

        // 1. Interpolation with provided named values
        let msg_custom = tr!("demo_i18n", "ru", "welcome_msg", name = "Player#1");
        assert_eq!(
            msg_custom,
            "^3[\x04GoldSrc.rs^3]^1 Добро пожаловать, \x04Player#1\x01!"
        );

        // 2. Interpolation with defaults (no params passed)
        let msg_default = tr!("demo_i18n", "ru", "welcome_msg");
        assert_eq!(
            msg_default,
            "^3[\x04GoldSrc.rs^3]^1 Добро пожаловать, \x04Гость\x01!"
        );

        // 3. Multi-param interpolation with numbers
        let reward = tr!("demo_i18n", "ru", "reward_msg", amount = 5000, xp = 250);
        assert_eq!(
            reward,
            "^3[\x04GoldSrc.rs^3]^1 Бонус: \x045000\x01 ₽ и \x04250\x01 XP!"
        );

        // 4. Test Player as AsLangCode in tr! macro
        let player = goldsrc_api::client::Player::new(1);
        assert_eq!(player.lang(), "en");
        let player_welcome = tr!("demo_i18n", &player, "welcome_msg", name = "TestUser");
        assert_eq!(
            player_welcome,
            "^3[\x04GoldSrc.rs^3]^1 Welcome, \x04TestUser\x01!"
        );

        // 5. Test server_lang()
        assert_eq!(I18nService::server_lang(), "en");
    }
}
