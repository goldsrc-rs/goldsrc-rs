//! Comprehensive test suite for the i18n subsystem.

use super::*;
use crate::tr;
use std::sync::Mutex;

/// Global test lock to serialize tests that mutate global `I18nEngine` state.
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

    // Test loading into I18nEngine
    I18nEngine::clear();
    let count = I18nEngine::load_toml_string("vip_menu", &toml_str).unwrap();
    assert_eq!(count, 4);

    let ru_reward =
        I18nEngine::translate("vip_menu", "ru", "money_reward", &[("amount", "5000")], &[]);
    assert_eq!(
        ru_reward,
        "^3[\x04VIP System^3]^1 Вам выдано: \x045000 ₽\x01!"
    );

    let en_reward =
        I18nEngine::translate("vip_menu", "en", "money_reward", &[("amount", "5000")], &[]);
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

    I18nEngine::clear();
    let count = I18nEngine::load_dir(&temp_dir);
    assert_eq!(count, 3);

    let title =
        I18nEngine::translate_with_caller("admin_system", "admin_system", "ru", "title", &[], &[]);
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

    I18nEngine::clear();
    let count = I18nEngine::load_toml_string("demo_i18n", toml_content).unwrap();
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
    assert_eq!(I18nEngine::server_lang(), "en");
}
