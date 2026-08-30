//! Demonstration plugin showcasing chat middleware, prefixing, and custom placeholder providers.

use goldsrc::chat::register_chat_middleware;
use goldsrc::log_info;
use goldsrc::placeholders::register_placeholder;
use goldsrc_macros::plugin;

pub struct TestChatPlugin;

#[plugin(
    name = "test_chat",
    version = "1.0.0",
    author = "GoldSrc.rs Team",
    description = "Demonstrates chat middleware and custom placeholders"
)]
impl TestChatPlugin {
    #[on_load]
    pub fn on_load() {
        log_info!("[test_chat] Initializing Chat & Placeholder Demo plugin...");

        // 1. Register custom placeholder: {server_tag}
        register_placeholder("server_tag", "Demo Server", |_, _| "GoldSrc.rs".to_string());

        // 2. Register custom placeholder: {kills}
        register_placeholder("kills", "Player kills counter", |caller, _| {
            format!("{}", caller.index * 5)
        });

        // 3. Register custom chat middleware prefixing VIP players
        register_chat_middleware(|msg| {
            if msg.sender.index == 1 {
                msg.prefix = Some("^4[Admin]^1 ".to_string());
            } else {
                msg.prefix = Some("^3[Player]^1 ".to_string());
            }
            true // continue processing
        });
    }
}
