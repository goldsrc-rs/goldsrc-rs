//! GoldSrc.rs Standard Communication & Chat Director Suite (`goldsrc:chat_director`).
//!
//! Provides antiflood rate limiting, private staff communication channels (`say_team @`),
//! and periodic rotating informational broadcasts across chat and Director HUD.

pub mod antiflood;
pub mod config;
pub mod error;
pub mod rotator;

use antiflood::AntifloodService;
use config::ChatDirectorConfig;
#[allow(unused_imports)]
use goldsrc::api::bindings::goldsrc::engine::api as host_api;
use goldsrc::prelude::*;
use goldsrc_api::timer::host_time;
use rotator::BroadcastRotator;
use std::sync::RwLock;

static CONFIG: RwLock<Option<ChatDirectorConfig>> = RwLock::new(None);
static ANTIFLOOD: RwLock<Option<AntifloodService>> = RwLock::new(None);
static ROTATOR: RwLock<Option<BroadcastRotator>> = RwLock::new(None);

pub struct ChatDirector;

pub mod caps {
    pub const CHAT_STAFF: &str = "admin:chat";
    pub const BROADCAST: &str = "chat:broadcast";
}

#[plugin(
    name = "chat_director",
    role = "service",
    bundle = "gameplay",
    version = "0.19.0",
    author = "GoldSrc.rs Team",
    description = "Antiflood protection, private staff communications, and rotating HUD broadcasts",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl ChatDirector {
    #[on_load]
    fn init() {
        log_info!("[Chat Director] Initializing communication service (v0.19.0)...");

        // 1. Register capabilities
        Auth::register_capability(
            caps::CHAT_STAFF,
            "Access to private staff communication channels",
        );
        Auth::register_capability(
            caps::BROADCAST,
            "Allows triggering immediate global server announcements",
        );

        // 2. Initialize configuration
        if let Ok(mut lock) = CONFIG.write() {
            *lock = Some(ChatDirectorConfig::default());
        }

        // 3. Initialize antiflood and rotator
        if let Ok(mut lock) = ANTIFLOOD.write() {
            *lock = Some(AntifloodService::new());
        }
        if let Ok(mut lock) = ROTATOR.write() {
            *lock = Some(BroadcastRotator::new());
        }

        // 4. Register chat middleware for Antiflood and say_team @ staff routing
        goldsrc::chat::register_chat_middleware(|msg| {
            let sender = msg.sender.index();
            let now = host_time();

            // Antiflood rate limit verification
            let flood_interval = if let Ok(lock) = CONFIG.read() {
                lock.as_ref().map(|c| c.flood_interval).unwrap_or(0.75)
            } else {
                0.75
            };

            if let Ok(mut lock) = ANTIFLOOD.write()
                && let Some(service) = lock.as_mut()
                && let Err(wait_secs) = service.check_and_update(sender, now, flood_interval)
            {
                msg.block();
                let warn_msg = tr!(
                    "chat_director",
                    &msg.sender,
                    "flood_warning",
                    wait = format!("{wait_secs:.1}")
                );
                msg.sender.print_notify(&warn_msg);
                return false;
            }

            // Staff channel interception: say_team @ <text> or say @ <text>
            let is_staff_call = msg.raw_text.trim().starts_with('@');
            if is_staff_call {
                let staff_content = msg
                    .raw_text
                    .trim()
                    .trim_start_matches('@')
                    .trim()
                    .to_string();
                let sender_name = msg
                    .sender
                    .name()
                    .unwrap_or_else(|| format!("Player #{sender}"));
                let formatted_staff_msg = format!("(STAFF CALL) {sender_name}: {staff_content}");

                msg.block(); // Prevent broadcast to regular public chat

                log_info!("[Staff Chat] {}", formatted_staff_msg);

                // Dispatch to online staff members possessing admin:chat capability
                for i in 1..=32 {
                    let p = Player::new(i);
                    if p.is_valid()
                        && (p.has_capability(caps::CHAT_STAFF)
                            || p.has_capability("moderation:inspect"))
                    {
                        p.print_chat(&formatted_staff_msg);
                    }
                }
                msg.sender
                    .print_chat(format!("(К ПЕРСОНАЛУ) {staff_content}"));
                return false;
            }

            true
        });

        log_info!("[Chat Director] Antiflood and staff channel filters registered successfully.");
    }

    #[on_frame]
    fn frame_tick() {
        let now = host_time();
        let (interval, enable_dhud) = if let Ok(lock) = CONFIG.read() {
            if let Some(c) = lock.as_ref() {
                (c.broadcast_interval, c.enable_dhud_banners)
            } else {
                (60.0, true)
            }
        } else {
            (60.0, true)
        };

        if let Ok(mut lock) = ROTATOR.write()
            && let Some(rotator) = lock.as_mut()
            && let Some(announcement) = rotator.check_tick(now, interval)
        {
            // 1. Broadcast to player chat
            chat_broadcast!(&announcement);

            // 2. Broadcast banner via Director HUD
            if enable_dhud {
                #[cfg(target_arch = "wasm32")]
                {
                    host_api::host_send_dhud_message(
                        -1, // Broadcast to all
                        0.05,
                        0.02, // Top-left position
                        0,
                        255,
                        200,
                        255, // Cyan color
                        0,   // Effect
                        0.5,
                        0.5,
                        5.0, // FadeIn, FadeOut, HoldTime
                        &announcement,
                    );
                }
            }
        }
    }

    // --- Command Implementations ---

    /// Dispatches an immediate high-priority server announcement to chat and screen.
    #[command(
        name = "grs_chat_broadcast",
        aliases = ["broadcast", "/broadcast"],
        capability = "chat:broadcast",
        description = "Broadcasts a high-priority announcement to all player chats and HUD",
        usage = "grs_chat_broadcast <message>"
    )]
    fn cmd_broadcast(message: String) {
        if message.trim().is_empty() {
            return;
        }
        let banner = format!("[ОБЪЯВЛЕНИЕ] {message}");
        chat_broadcast!(&banner);

        #[cfg(target_arch = "wasm32")]
        {
            host_api::host_send_dhud_message(
                -1, -1.0, 0.20, // Center-top
                255, 180, 0, 255, // Gold
                0, 0.2, 0.5, 6.0, &banner,
            );
        }

        log_info!(
            "[Chat Director] Dispatched broadcast announcement: {}",
            message
        );
    }

    /// Sends a direct message to all online staff.
    #[command(
        name = "admin_chat",
        aliases = ["ac", "staffchat"],
        capability = "admin:chat",
        description = "Sends a message in the private administrative channel",
        usage = "admin_chat <message>"
    )]
    fn cmd_admin_chat(player: Player, message: String) {
        let sender_name = player
            .name()
            .unwrap_or_else(|| format!("Player #{}", player.index()));
        let formatted = format!("(ADMIN CHAT) {sender_name}: {message}");

        for i in 1..=32 {
            let p = Player::new(i);
            if p.is_valid()
                && (p.has_capability(caps::CHAT_STAFF) || p.has_capability("moderation:inspect"))
            {
                p.print_chat(&formatted);
            }
        }
        log_info!("[Chat Director] {}", formatted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_director_config_derives_toml() {
        let cfg = ChatDirectorConfig::default();
        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("flood_interval = 0.75"));
        assert!(toml_str.contains("broadcast_interval = 60"));
        let cvars = cfg.to_cvars();
        assert!(cvars.contains("grs_chat_flood_interval"));
    }

    #[test]
    fn test_antiflood_rate_limiter() {
        let mut af = AntifloodService::new();
        // First message at t=1.0 is OK
        assert!(af.check_and_update(1, 1.0, 0.75).is_ok());
        // Second message at t=1.2 is blocked (<0.75s)
        assert!(af.check_and_update(1, 1.2, 0.75).is_err());
        // Third message at t=2.0 is OK (>0.75s)
        assert!(af.check_and_update(1, 2.0, 0.75).is_ok());
    }

    #[test]
    fn test_broadcast_rotator_timing() {
        let mut rotator = BroadcastRotator::new();
        // At t=10, None (interval not elapsed)
        assert!(rotator.check_tick(10.0, 60.0).is_none());
        // At t=60, first message dispatched
        assert!(rotator.check_tick(60.0, 60.0).is_some());
        // At t=70, None
        assert!(rotator.check_tick(70.0, 60.0).is_none());
        // At t=121, next message dispatched
        assert!(rotator.check_tick(121.0, 60.0).is_some());
    }
}
