//! GoldSrc.rs Standard Map Manager Suite (`goldsrc:map_manager`).
//!
//! Provides automatic timelimit monitoring, player nominations, interactive vote ballots,
//! anti-repeat map protection, and nextmap rotation.

pub mod config;
pub mod error;
pub mod menu;
pub mod tracker;
pub mod vote;

use config::MapManagerConfig;
#[allow(unused_imports)]
use error::MapManagerError;
#[allow(unused_imports)]
use goldsrc::api::bindings::goldsrc::engine::api as host_api;
use goldsrc::prelude::*;
use goldsrc_api::timer::host_time;
use menu::*;
use std::sync::RwLock;
use tracker::MapTracker;
use vote::MapVoteSession;

static CONFIG: RwLock<Option<MapManagerConfig>> = RwLock::new(None);
static TRACKER: RwLock<Option<MapTracker>> = RwLock::new(None);
static VOTE: RwLock<Option<MapVoteSession>> = RwLock::new(None);

pub struct MapManager;

pub mod caps {
    pub const VOTE_START: &str = "map:vote";
    pub const NOMINATE: &str = "map:nominate";
}

#[plugin(
    name = "map_manager",
    role = "feature",
    bundle = "gameplay",
    version = "0.19.0",
    author = "GoldSrc.rs Team",
    description = "Map timelimit tracker, public nominations, interactive voting, and nextmap rotation",
    url = "https://github.com/goldsrc-rs/goldsrc-rs"
)]
impl MapManager {
    #[on_load]
    fn init() {
        log_info!("[Map Manager] Initializing map rotation and voting suite (v0.19.0)...");

        // 1. Register capabilities
        Auth::register_capability(
            caps::VOTE_START,
            "Allows manually initiating map rotation votes",
        );
        Auth::register_capability(
            caps::NOMINATE,
            "Allows players to nominate maps for the next ballot",
        );

        // 2. Initialize configuration
        if let Ok(mut lock) = CONFIG.write() {
            *lock = Some(MapManagerConfig::default());
        }

        // 3. Initialize tracker and voting session
        let now = host_time();
        if let Ok(mut lock) = TRACKER.write() {
            *lock = Some(MapTracker::new(now));
        }
        if let Ok(mut lock) = VOTE.write() {
            *lock = Some(MapVoteSession::default());
        }

        log_info!("[Map Manager] Timelimit tracker and vote engine online.");
    }

    #[on_frame]
    fn frame_tick() {
        let now = host_time();

        // 1. Check if vote should be automatically triggered based on mp_timelimit
        let should_start_vote = if let (Ok(t_lock), Ok(v_lock), Ok(c_lock)) =
            (TRACKER.read(), VOTE.read(), CONFIG.read())
        {
            if let (Some(tracker), Some(vote), Some(config)) =
                (t_lock.as_ref(), v_lock.as_ref(), c_lock.as_ref())
            {
                if !vote.is_active && !vote.is_concluded {
                    if let Some(left) = tracker.time_left(now) {
                        left <= (config.vote_trigger_mins * 60.0)
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if should_start_vote {
            Self::trigger_map_vote(now);
        }

        // 2. Check if active vote concluded
        let concluded_winner = if let Ok(mut lock) = VOTE.write() {
            if let Some(vote) = lock.as_mut() {
                vote.check_conclude(now)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(winner) = concluded_winner {
            chat_broadcast!(&format!(
                "[Map Manager] Голосование завершено! Следующая карта: {winner}"
            ));

            #[cfg(target_arch = "wasm32")]
            {
                host_api::host_send_dhud_message(
                    -1,
                    -1.0,
                    0.35, // Center
                    0,
                    255,
                    120,
                    255, // Green
                    0,
                    0.2,
                    0.5,
                    7.0,
                    format!("Следующая карта: {winner}"),
                );
            }

            if let Ok(mut lock) = TRACKER.write()
                && let Some(tracker) = lock.as_mut()
            {
                tracker.next_map = Some(winner.clone());
            }

            log_info!(
                "[Map Manager] Concluded map vote: winner is '{}'. Setting amx_nextmap and preparing rotation.",
                winner
            );
            server_command(format!("amx_nextmap \"{}\"\n", winner));
        }
    }

    /// Triggers an interactive map vote across all connected players.
    pub fn trigger_map_vote(now: f32) {
        let (options, duration) = if let Ok(lock) = CONFIG.read() {
            if let Some(cfg) = lock.as_ref() {
                let pool: Vec<String> = cfg
                    .default_pool
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .take(5)
                    .collect();
                (pool, cfg.vote_duration_secs as f32)
            } else {
                (vec!["de_dust2".to_string(), "de_inferno".to_string()], 15.0)
            }
        } else {
            (vec!["de_dust2".to_string(), "de_inferno".to_string()], 15.0)
        };

        if let Ok(mut lock) = VOTE.write()
            && let Some(vote) = lock.as_mut()
        {
            vote.start(options.clone(), now, duration);
        }

        chat_broadcast!("[Map Manager] Внимание! Открыто голосование за следующую карту!");

        // Open voting menu for all active players
        let ballot = build_vote_menu(&options);
        for i in 1..=32 {
            let p = Player::new(i);
            if p.is_valid() {
                p.open_menu(&ballot);
            }
        }

        log_info!(
            "[Map Manager] Started map vote with {} candidates ({:.0}s)",
            options.len(),
            duration
        );
    }

    // --- Command Implementations ---

    /// Reports time remaining until map rotation.
    #[command(
        name = "grs_timeleft",
        aliases = ["timeleft", "/timeleft"],
        description = "Displays the remaining time for the current map",
        usage = "grs_timeleft"
    )]
    fn cmd_timeleft() {
        let now = host_time();
        let formatted = if let Ok(lock) = TRACKER.read() {
            lock.as_ref()
                .map(|t| t.format_time_left(now))
                .unwrap_or_else(|| "N/A".to_string())
        } else {
            "N/A".to_string()
        };

        chat_broadcast!(&format!(
            "[Map Manager] До конца карты осталось: {formatted}"
        ));
    }

    /// Displays the currently active map.
    #[command(
        name = "grs_currentmap",
        aliases = ["currentmap", "/currentmap"],
        description = "Displays the current active map",
        usage = "grs_currentmap"
    )]
    fn cmd_currentmap() {
        let current = if let Ok(lock) = TRACKER.read() {
            lock.as_ref()
                .map(|t| t.current_map.clone())
                .unwrap_or_else(|| "crossfire".to_string())
        } else {
            "crossfire".to_string()
        };
        chat_broadcast!(&format!("[Map Manager] Текущая карта: {current}"));
    }

    /// Displays the chosen next map in rotation.
    #[command(
        name = "grs_nextmap",
        aliases = ["nextmap", "/nextmap"],
        description = "Displays the chosen next map for the upcoming rotation",
        usage = "grs_nextmap"
    )]
    fn cmd_nextmap() {
        let next = if let Ok(lock) = TRACKER.read() {
            lock.as_ref()
                .and_then(|t| t.next_map.clone())
                .unwrap_or_else(|| "Ещё не выбрана (ожидается голосование)".to_string())
        } else {
            "Ещё не выбрана".to_string()
        };
        chat_broadcast!(&format!("[Map Manager] Следующая карта: {next}"));
    }

    /// Opens the interactive nomination menu.
    #[command(
        name = "grs_maps",
        aliases = ["maps", "/maps"],
        description = "Opens the map nomination menu for the upcoming vote",
        usage = "grs_maps"
    )]
    fn cmd_maps(player: Player) {
        if !player.is_valid() {
            return;
        }

        let maps = if let Ok(lock) = CONFIG.read() {
            lock.as_ref()
                .map(|c| {
                    c.default_pool
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let menu = build_nomination_menu(&maps);
        player.open_menu(&menu);
    }

    /// Manually triggers map vote (admin command).
    #[command(
        name = "grs_mapvote_start",
        capability = "map:vote",
        description = "Forces immediate start of the map voting session",
        usage = "grs_mapvote_start"
    )]
    fn cmd_mapvote_start() {
        Self::trigger_map_vote(host_time());
    }

    // --- Menu Action Handlers for Voting Ballot ---

    #[menu_action(id = 6000)]
    fn on_vote_opt_0(player: &mut Player) {
        Self::record_player_vote(player, 0);
    }

    #[menu_action(id = 6001)]
    fn on_vote_opt_1(player: &mut Player) {
        Self::record_player_vote(player, 1);
    }

    #[menu_action(id = 6002)]
    fn on_vote_opt_2(player: &mut Player) {
        Self::record_player_vote(player, 2);
    }

    #[menu_action(id = 6003)]
    fn on_vote_opt_3(player: &mut Player) {
        Self::record_player_vote(player, 3);
    }

    #[menu_action(id = 6004)]
    fn on_vote_opt_4(player: &mut Player) {
        Self::record_player_vote(player, 4);
    }

    fn record_player_vote(player: &mut Player, option_idx: usize) {
        let success = if let Ok(mut lock) = VOTE.write() {
            if let Some(v) = lock.as_mut() {
                v.cast_vote(player.index(), option_idx)
            } else {
                false
            }
        } else {
            false
        };

        if success {
            player.print_center("[Голосование] Ваш голос успешно учтен!");
        } else {
            player.print_center("[Голосование] Голосование неактивно или неверный пункт.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_manager_config_derives_toml() {
        let cfg = MapManagerConfig::default();
        let toml_str = cfg.to_toml();
        assert!(toml_str.contains("vote_trigger_mins = 2.5"));
        assert!(toml_str.contains("vote_duration_secs = 15"));
        let cvars = cfg.to_cvars();
        assert!(cvars.contains("grs_map_vote_trigger_mins"));
    }

    #[test]
    fn test_map_tracker_format_time_left() {
        let tracker = MapTracker::new(0.0);
        // Explicit 0 timelimit -> Без ограничений
        assert_eq!(
            tracker.format_time_left_with_limit(100.0, 0.0),
            "Без ограничений"
        );
        // Explicit 20 mins timelimit -> (20 * 60 - 100) = 1100s -> 18:20
        assert_eq!(tracker.format_time_left_with_limit(100.0, 20.0), "18:20");
    }

    #[test]
    fn test_map_vote_session_tally() {
        let mut vote = MapVoteSession::default();
        let options = vec!["de_dust2".to_string(), "de_inferno".to_string()];
        vote.start(options, 0.0, 15.0);

        // Player 1 and 2 vote for de_inferno (opt 1), Player 3 votes for de_dust2 (opt 0)
        vote.cast_vote(1, 1);
        vote.cast_vote(2, 1);
        vote.cast_vote(3, 0);

        // Check before duration -> None
        assert!(vote.check_conclude(10.0).is_none());

        // Check after duration -> Some("de_inferno")
        let winner = vote.check_conclude(16.0);
        assert_eq!(winner, Some("de_inferno".to_string()));
    }
}
