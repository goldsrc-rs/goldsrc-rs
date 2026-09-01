//! Demonstration plugin showcasing chat middleware, prefixing, and custom placeholder providers.

use goldsrc::chat::register_chat_middleware;
use goldsrc::placeholders::register_placeholder;
use goldsrc::prelude::*;

pub struct TestChatPlugin;

#[plugin(
    name = "test_chat",
    version = "1.0.0",
    author = "GoldSrc.rs Team",
    description = "Demonstrates chat middleware, chunking, and custom placeholders"
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

        // 3. Register long text placeholder to test automatic multi-packet chunk splitting (>180 bytes)
        register_placeholder(
            "long_text",
            "Returns a long text payload for chunk testing",
            |_, _| {
                "^3[GoldSrc.rs Long Text Demo]^1: This is an extended message designed to test automatic 180-byte chunk splitting in the chat engine. It demonstrates that long text spans across multiple SayText packets safely without crashing the client or losing color styling!".to_string()
            },
        );

        // 4. Register custom chat middleware prefixing players
        register_chat_middleware(|msg| {
            if msg.sender.index == 1 {
                msg.prefix = Some("^4[Admin]^1 ".to_string());
            } else {
                msg.prefix = Some("^3[Player]^1 ".to_string());
            }
            true // continue processing
        });
    }

    /// Test command demonstrating chat macros and long text chunking.
    #[command(
        name = "testchat",
        aliases = ["/testchat", "!testchat"],
        description = "Tests chat formatting, placeholders, and long text chunking"
    )]
    fn handle_test_chat(caller: i32) {
        let player = Player::new(caller);
        chat_print!(
            player,
            "^4[test_chat]^1 Hello {name}! Your tag: {server_tag}, kills: {kills}"
        );
        chat_print!(player, "{long_text}");
    }
}
