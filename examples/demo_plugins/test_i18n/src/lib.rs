use goldsrc::prelude::*;

pub struct TestI18n;

#[plugin(
    name = "test_i18n",
    version = "0.15.0",
    author = "GoldSrc.rs Team",
    description = "Multilingual i18n & Localization verification plugin for GoldSrc.rs",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl TestI18n {
    #[on_load]
    fn init() {
        log_info!("[test_i18n] Localization verification plugin v0.15.0 loaded.");
    }

    /// Sends localized multi-language messages to a player.
    /// Usage: `test_lang <player_index> <ru|en|es|de|unknown>`
    #[command(
        name = "test_lang",
        aliases = ["langtest", "/lang", "!lang"],
        description = "Tests dictionary translations in ru, en, es, de languages + fallback chain",
        usage = "test_lang <player_index> <lang_code>"
    )]
    fn test_language(target: Player, lang: String) {
        let lang_code = lang.to_lowercase();
        let target_idx = target.index();
        let player_name = target
            .name()
            .unwrap_or_else(|| format!("Player#{target_idx}"));

        // 1. Format welcome message using tr! macro with named placeholder and explicit lang_code
        let welcome = tr!("test_i18n", &lang_code, "welcome_msg", name = player_name);

        // 2. Format reward message using tr! macro with multiple parameters & defaults
        let reward = tr!(
            "test_i18n",
            &lang_code,
            "reward_claimed",
            amount = 5000,
            xp = 250
        );

        // 3. Format admin alert
        let alert = tr!(
            "test_i18n",
            &lang_code,
            "admin_alert",
            admin = "ServerAdmin",
            map = "de_dust2"
        );

        // 4. Test player-based language resolution (&target implementing AsLangCode)
        let player_default_msg = tr!("test_i18n", &target, "welcome_msg");

        // 5. Test multi-level fallback to system 'common' dictionary
        let btn_confirm = tr!("test_i18n", &lang_code, "btn_yes");
        let btn_cancel = tr!("test_i18n", &lang_code, "btn_no");

        // Send messages to client
        target.print_chat(&welcome);
        target.print_chat(&reward);
        target.print_chat(&alert);
        target.print_chat(&format!(
            "^3[Player Lang ({} Defaults)]^1 {}",
            target.lang(),
            player_default_msg
        ));
        target.print_chat(&format!(
            "^3[Common Fallback]^1 OK: \x04{btn_confirm}\x01 | Cancel: \x04{btn_cancel}\x01"
        ));

        log_info!(
            "[test_i18n] Dispatched '{}' messages to player #{}:\n  {}\n  {}\n  {}\n  Player Default ({}): {}\n  Buttons: {} / {}",
            lang_code,
            target_idx,
            welcome,
            reward,
            alert,
            target.lang(),
            player_default_msg,
            btn_confirm,
            btn_cancel
        );
    }
}
