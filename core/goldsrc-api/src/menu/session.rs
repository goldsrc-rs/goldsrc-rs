//! Runtime Menu Session state and navigation router.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::menu::{
    ExitBehavior, Menu, MenuContext, MenuRendererKind, RenderedMenuPage, SlotAction,
};

/// Active menu session for a player.
#[derive(Clone)]
pub struct PlayerMenuSession {
    pub menu: Menu,
    pub current_page: usize,
    pub history_stack: Vec<(Menu, usize)>,
    pub rendered_page: Option<RenderedMenuPage>,
}

type ParentMenuState = (Menu, usize, Vec<(Menu, usize)>);

static MENU_SESSIONS: std::sync::LazyLock<Mutex<HashMap<i32, PlayerMenuSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static PENDING_PARENT: std::sync::LazyLock<Mutex<HashMap<i32, ParentMenuState>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static MENU_DEBOUNCE_TRACKER: std::sync::LazyLock<Mutex<HashMap<i32, std::time::Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

static ITEM_COOLDOWN_TRACKER: std::sync::LazyLock<Mutex<HashMap<(i32, u32), std::time::Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Opens a declarative `Menu` for a player and registers its navigation session.
pub fn open_menu(player_idx: i32, menu: Menu) {
    let mut sessions = MENU_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());

    // Check if an active session or a pending parent from an action exists
    let pending = {
        let mut pend = PENDING_PARENT.lock().unwrap_or_else(|e| e.into_inner());
        pend.remove(&player_idx)
    };

    let history = if let Some((parent_menu, parent_page, parent_history)) = pending {
        let mut stack = parent_history;
        stack.push((parent_menu, parent_page));
        stack
    } else if let Some(existing) = sessions.remove(&player_idx) {
        let mut stack = existing.history_stack;
        stack.push((existing.menu, existing.current_page));
        stack
    } else {
        Vec::new()
    };

    let mut session = PlayerMenuSession {
        menu,
        current_page: 0,
        history_stack: history,
        rendered_page: None,
    };

    render_and_send(player_idx, &mut session);
    sessions.insert(player_idx, session);
}

/// Dispatches a selected slot (1..=10) for the player.
/// Returns `Some(SlotAction)` if an actionable execution occurred, or `None` if consumed by pagination/navigation.
pub fn handle_menu_slot(player_idx: i32, slot: u8) -> Option<SlotAction> {
    let mut sessions = MENU_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    let mut session = sessions.remove(&player_idx)?;

    let action = session.rendered_page.as_ref()?.slots.get(&slot)?.clone();

    match action {
        SlotAction::Execute {
            id,
            action_name,
            keep_open,
        } => {
            // 1. Menu global debounce anti-flood check
            if let Some(debounce_dur) = session.menu.debounce {
                let mut last_presses = MENU_DEBOUNCE_TRACKER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let now = std::time::Instant::now();
                if let Some(last) = last_presses.get(&player_idx)
                    && now.duration_since(*last) < debounce_dur
                {
                    sessions.insert(player_idx, session);
                    return None;
                }
                last_presses.insert(player_idx, now);
            }

            // 2. Item-level cooldown check
            if let Some(item) = session.menu.items.iter().find(|i| match i.kind {
                crate::menu::ItemKind::Action { id: item_id, .. } => item_id == id,
                _ => false,
            }) && let Some((cooldown_dur, ref on_spam)) = item.cooldown
            {
                let mut cooldowns = ITEM_COOLDOWN_TRACKER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let now = std::time::Instant::now();
                if let Some(last) = cooldowns.get(&(player_idx, id))
                    && now.duration_since(*last) < cooldown_dur
                {
                    match on_spam {
                        crate::menu::AntiSpamAction::Ignore
                        | crate::menu::AntiSpamAction::MakeInactive => {}
                        crate::menu::AntiSpamAction::Feedback(fb) => {
                            let player = crate::client::Player::new(player_idx);
                            if let Some((target, ref msg)) = fb.message {
                                player.print(target, msg);
                            }
                            if let Some(ref sound) = fb.sound {
                                player.play_sound(sound);
                            }
                        }
                        crate::menu::AntiSpamAction::CloseMenu => {
                            crate::client::Player::new(player_idx).show_raw_menu(0, 0, "");
                            return None;
                        }
                    }
                    sessions.insert(player_idx, session);
                    return None;
                }
                cooldowns.insert((player_idx, id), now);
            }

            // Save parent session state in case action handler calls open_menu
            if let Ok(mut pend) = PENDING_PARENT.lock() {
                pend.insert(
                    player_idx,
                    (
                        session.menu.clone(),
                        session.current_page,
                        session.history_stack.clone(),
                    ),
                );
            }

            if keep_open {
                // Re-render and restore session
                render_and_send(player_idx, &mut session);
                sessions.insert(player_idx, session);
            }

            // Drop sessions lock BEFORE returning action (so action handler can call open_menu without deadlocking)
            drop(sessions);

            if !keep_open {
                // If not keep_open, close client menu (if action opens a submenu, open_menu will replace it)
                crate::client::Player::new(player_idx).show_raw_menu(0, 0, "");
            }

            Some(SlotAction::Execute {
                id,
                action_name,
                keep_open,
            })
        }
        SlotAction::PrevPage => {
            if session.current_page > 0 {
                session.current_page -= 1;
            }
            render_and_send(player_idx, &mut session);
            sessions.insert(player_idx, session);
            None
        }
        SlotAction::NextPage => {
            session.current_page += 1;
            render_and_send(player_idx, &mut session);
            sessions.insert(player_idx, session);
            None
        }
        SlotAction::Exit => {
            match session.menu.exit_behavior {
                ExitBehavior::CloseAll => {
                    crate::client::Player::new(player_idx).show_raw_menu(0, 0, "");
                }
                ExitBehavior::PopParent => {
                    if let Some((parent_menu, parent_page)) = session.history_stack.pop() {
                        session.menu = parent_menu;
                        session.current_page = parent_page;
                        render_and_send(player_idx, &mut session);
                        sessions.insert(player_idx, session);
                    } else {
                        crate::client::Player::new(player_idx).show_raw_menu(0, 0, "");
                    }
                }
                ExitBehavior::PopParentPage(target_page) => {
                    if let Some((parent_menu, _)) = session.history_stack.pop() {
                        let total_pages = {
                            let player = crate::client::Player::new(player_idx);
                            let ctx = MenuContext {
                                player_index: player_idx,
                                round_number: 1,
                                round_time_elapsed: 0.0,
                                is_alive: player.health() > 0.0,
                                players_count: 1,
                            };
                            parent_menu
                                .render_page(&ctx, 0)
                                .map(|p| p.total_pages)
                                .unwrap_or(1)
                        };

                        let resolved_page = if target_page < 0 {
                            total_pages.saturating_sub(1)
                        } else {
                            (target_page as usize)
                                .saturating_sub(1)
                                .min(total_pages.saturating_sub(1))
                        };

                        session.menu = parent_menu;
                        session.current_page = resolved_page;
                        render_and_send(player_idx, &mut session);
                        sessions.insert(player_idx, session);
                    } else {
                        crate::client::Player::new(player_idx).show_raw_menu(0, 0, "");
                    }
                }
            }
            None
        }
        SlotAction::Noop => {
            sessions.insert(player_idx, session);
            None
        }
        SlotAction::DenyFeedback(ref deny_act) => {
            if let crate::menu::DenyAction::Feedback(fb) = deny_act {
                let player = crate::client::Player::new(player_idx);
                if let Some((target, ref msg)) = fb.message {
                    player.print(target, msg);
                }
                if let Some(ref sound) = fb.sound {
                    player.play_sound(sound);
                }
            }
            sessions.insert(player_idx, session);
            None
        }
    }
}

/// Clears menu session, debounces, and cooldowns for the player.
pub fn close_menu(player_idx: i32) {
    if let Ok(mut sessions) = MENU_SESSIONS.lock()
        && sessions.remove(&player_idx).is_some()
    {
        crate::client::Player::new(player_idx).show_raw_menu(0, 0, "");
    }
    if let Ok(mut pending) = PENDING_PARENT.lock() {
        pending.remove(&player_idx);
    }
    if let Ok(mut debounce) = MENU_DEBOUNCE_TRACKER.lock() {
        debounce.remove(&player_idx);
    }
    if let Ok(mut cooldowns) = ITEM_COOLDOWN_TRACKER.lock() {
        cooldowns.retain(|(p, _), _| *p != player_idx);
    }
}

/// Closes all menu sessions and purges all tracker maps on map change / shutdown.
pub fn clear_all_menus() {
    if let Ok(mut sessions) = MENU_SESSIONS.lock() {
        sessions.clear();
    }
    if let Ok(mut pending) = PENDING_PARENT.lock() {
        pending.clear();
    }
    if let Ok(mut debounce) = MENU_DEBOUNCE_TRACKER.lock() {
        debounce.clear();
    }
    if let Ok(mut cooldowns) = ITEM_COOLDOWN_TRACKER.lock() {
        cooldowns.clear();
    }
}

/// Re-renders and sends the currently open menu for `player_idx` if present, updating navigation text for their active language.
pub fn refresh_player_menu(player_idx: i32) {
    let mut sessions = MENU_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(session) = sessions.get_mut(&player_idx) {
        let player = crate::client::Player::new(player_idx);
        let lang = player.lang();
        session.menu.style = session.menu.style.clone().with_lang(&lang);
        render_and_send(player_idx, session);
    }
}

/// Re-renders and sends all currently open menus for all players.
pub fn refresh_all_menus() {
    let mut sessions = MENU_SESSIONS.lock().unwrap_or_else(|e| e.into_inner());
    for (&player_idx, session) in sessions.iter_mut() {
        let player = crate::client::Player::new(player_idx);
        let lang = player.lang();
        session.menu.style = session.menu.style.clone().with_lang(&lang);
        render_and_send(player_idx, session);
    }
}

fn render_and_send(player_idx: i32, session: &mut PlayerMenuSession) {
    let player = crate::client::Player::new(player_idx);
    let ctx = MenuContext {
        player_index: player_idx,
        round_number: 1,
        round_time_elapsed: 0.0,
        is_alive: player.health() > 0.0,
        players_count: 1,
    };

    if let Some(rendered) = session.menu.render_page(&ctx, session.current_page) {
        match rendered.renderer {
            MenuRendererKind::Text => {
                player.show_raw_menu(rendered.keys_mask as i32, rendered.timeout, &rendered.text);
            }
            MenuRendererKind::Dhud {
                position,
                color,
                effect,
            } => {
                let hud_msg = crate::hud::HudMessage {
                    text: rendered.text.clone(),
                    kind: crate::hud::HudKind::Dhud,
                    color,
                    color2: color,
                    position,
                    effect,
                };
                player.send_hud(&hud_msg);
                player.show_raw_menu(rendered.keys_mask as i32, rendered.timeout, "");
            }
        }
        session.rendered_page = Some(rendered);
    }
}
