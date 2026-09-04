//! Core domain types for the interactive GoldSrc.rs Menu System.

use std::sync::Arc;

/// Context passed to dynamic menu item formatters and conditions during page evaluation.
#[derive(Debug, Clone)]
pub struct MenuContext {
    pub player_index: i32,
    pub round_number: u32,
    pub round_time_elapsed: f32,
    pub is_alive: bool,
    pub players_count: u32,
}

impl MenuContext {
    pub fn new(player_index: i32) -> Self {
        Self {
            player_index,
            round_number: 1,
            round_time_elapsed: 0.0,
            is_alive: true,
            players_count: 1,
        }
    }
}

/// Type alias for dynamic deny format functions.
pub type DenyFormatFn = Arc<dyn Fn(&str, &str) -> String + Send + Sync>;
/// Type alias for custom deny action callbacks.
pub type DenyActionFn = Arc<dyn Fn(i32) + Send + Sync>;
/// Type alias for dynamic condition predicates.
pub type ConditionFn = Arc<dyn Fn(&MenuContext) -> Result<(), String> + Send + Sync>;
/// Type alias for dynamic item title resolvers.
pub type DynamicTitleFn = Arc<dyn Fn(&MenuContext) -> String + Send + Sync>;
/// Type alias for header format functions.
pub type HeaderFormatFn = Arc<dyn Fn(&str, usize, usize) -> String + Send + Sync>;
/// Type alias for item format functions.
pub type ItemFormatFn = Arc<dyn Fn(usize, &str) -> String + Send + Sync>;

/// Visual presentation policy when a menu item's condition fails.
#[derive(Clone)]
pub enum VisualDeny {
    /// Item is formatted with dim/grey color `\d` (e.g. `\d1. Buy AWP`).
    Dimmed,
    /// Item title is completely replaced (e.g. `\d1. [Unavailable until Round 3]`).
    Replace(String),
    /// Dynamic formatting function receiving original title and failure reason.
    Format(DenyFormatFn),
    /// Item is completely omitted from the page, shifting subsequent items up.
    Hide,
}

impl std::fmt::Debug for VisualDeny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dimmed => write!(f, "VisualDeny::Dimmed"),
            Self::Replace(s) => write!(f, "VisualDeny::Replace({s:?})"),
            Self::Format(_) => write!(f, "VisualDeny::Format(<fn>)"),
            Self::Hide => write!(f, "VisualDeny::Hide"),
        }
    }
}

/// Unified user feedback notification (message directed to a PrintTarget and/or audio sound).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Feedback {
    pub message: Option<(crate::client::PrintTarget, String)>,
    pub sound: Option<String>,
}

impl Feedback {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn center(msg: impl Into<String>) -> Self {
        Self {
            message: Some((crate::client::PrintTarget::Center, msg.into())),
            sound: None,
        }
    }

    pub fn chat(msg: impl Into<String>) -> Self {
        Self {
            message: Some((crate::client::PrintTarget::ColoredChat, msg.into())),
            sound: None,
        }
    }

    pub fn console(msg: impl Into<String>) -> Self {
        Self {
            message: Some((crate::client::PrintTarget::Console, msg.into())),
            sound: None,
        }
    }

    pub fn notify(msg: impl Into<String>) -> Self {
        Self {
            message: Some((crate::client::PrintTarget::Notify, msg.into())),
            sound: None,
        }
    }

    pub fn sound(mut self, sound_path: impl Into<String>) -> Self {
        self.sound = Some(sound_path.into());
        self
    }

    pub fn message(mut self, target: crate::client::PrintTarget, text: impl Into<String>) -> Self {
        self.message = Some((target, text.into()));
        self
    }
}

impl From<&str> for Feedback {
    fn from(s: &str) -> Self {
        Self::center(s)
    }
}

impl From<String> for Feedback {
    fn from(s: String) -> Self {
        Self::center(s)
    }
}

/// Action taken when a player activates a menu item that is currently on cooldown.
#[derive(Debug, Clone, Default)]
pub enum AntiSpamAction {
    /// Silently ignore key press. Default.
    #[default]
    Ignore,
    /// Keep the item visually inactive / disabled while cooldown is active.
    MakeInactive,
    /// Deliver custom feedback to the player (center, chat, console, notify and/or sound).
    Feedback(Feedback),
    /// Close the menu immediately.
    CloseMenu,
}

impl From<Feedback> for AntiSpamAction {
    fn from(fb: Feedback) -> Self {
        Self::Feedback(fb)
    }
}

/// Behavioral response when an unauthorized/denied menu slot is pressed.
#[derive(Clone, Default)]
pub enum DenyAction {
    /// Slot is excluded from `ShowMenu` keys mask; pressing key is ignored by engine.
    #[default]
    Disabled,
    /// Slot is included in keys mask; pressing key triggers no-op and keeps menu open.
    Noop,
    /// Slot is included in keys mask; pressing key dispatches custom feedback message/sound.
    Feedback(Feedback),
    /// Custom callback function executed when player presses the denied slot.
    Custom(DenyActionFn),
}

impl DenyAction {
    pub fn feedback(feedback: impl Into<Feedback>) -> Self {
        Self::Feedback(feedback.into())
    }
}

impl From<Feedback> for DenyAction {
    fn from(fb: Feedback) -> Self {
        Self::Feedback(fb)
    }
}

impl std::fmt::Debug for DenyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "DenyAction::Disabled"),
            Self::Noop => write!(f, "DenyAction::Noop"),
            Self::Feedback(fb) => write!(f, "DenyAction::Feedback({fb:?})"),
            Self::Custom(_) => write!(f, "DenyAction::Custom(<fn>)"),
        }
    }
}

/// Composite deny policy governing visual rendering and interaction on condition failure.
#[derive(Debug, Clone)]
pub struct DenyPolicy {
    pub visual: VisualDeny,
    pub action: DenyAction,
}

impl DenyPolicy {
    /// Default disabled policy: Dimmed grey text, non-clickable slot.
    pub const fn disabled() -> Self {
        Self {
            visual: VisualDeny::Dimmed,
            action: DenyAction::Disabled,
        }
    }

    /// Hidden policy: Completely omitted from the rendered page.
    pub const fn hide() -> Self {
        Self {
            visual: VisualDeny::Hide,
            action: DenyAction::Disabled,
        }
    }

    /// Replace policy: Replaces title with alternative text, non-clickable slot.
    pub fn replace<S: Into<String>>(new_title: S) -> Self {
        Self {
            visual: VisualDeny::Replace(new_title.into()),
            action: DenyAction::Disabled,
        }
    }

    /// Interactive feedback policy: Shows replaced title and emits sound/message on click.
    pub fn feedback<S: Into<String>>(title: S, feedback_msg: S, sound: Option<String>) -> Self {
        Self {
            visual: VisualDeny::Replace(title.into()),
            action: DenyAction::Feedback(Feedback {
                message: Some((crate::client::PrintTarget::Center, feedback_msg.into())),
                sound,
            }),
        }
    }
}

impl Default for DenyPolicy {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Dynamic condition predicate evaluated before displaying or selecting a menu item.
#[derive(Clone)]
pub enum Condition {
    /// Requires a specific capability via `Auth::has_capability`.
    Capability(String),
    /// Accessible only starting from a minimum round number.
    MinRound(u32),
    /// Accessible only if within elapsed seconds of round start.
    TimeLimit(f32),
    /// Accessible only if server has at least `N` players.
    MinPlayers(u32),
    /// Accessible only to alive players.
    AliveOnly,
    /// Accessible only to dead players / spectators.
    DeadOnly,
    /// Custom predicate returning `Ok(())` or `Err(Reason)`.
    Custom(ConditionFn),
}

impl Condition {
    pub fn check(&self, ctx: &MenuContext) -> Result<(), String> {
        match self {
            Self::Capability(cap) => {
                if crate::auth::Auth::has_capability(ctx.player_index, cap) {
                    Ok(())
                } else {
                    Err(format!("Requires capability '{cap}'"))
                }
            }
            Self::MinRound(r) => {
                if ctx.round_number >= *r {
                    Ok(())
                } else {
                    Err(format!("Available from round {r}"))
                }
            }
            Self::TimeLimit(t) => {
                if ctx.round_time_elapsed <= *t {
                    Ok(())
                } else {
                    Err(format!("Expired (limit: {t:.0}s)"))
                }
            }
            Self::MinPlayers(p) => {
                if ctx.players_count >= *p {
                    Ok(())
                } else {
                    Err(format!("Requires at least {p} players"))
                }
            }
            Self::AliveOnly => {
                if ctx.is_alive {
                    Ok(())
                } else {
                    Err("Alive players only".into())
                }
            }
            Self::DeadOnly => {
                if !ctx.is_alive {
                    Ok(())
                } else {
                    Err("Dead players only".into())
                }
            }
            Self::Custom(cb) => cb(ctx),
        }
    }
}

impl std::fmt::Debug for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Capability(c) => write!(f, "Condition::Capability({c:?})"),
            Self::MinRound(r) => write!(f, "Condition::MinRound({r})"),
            Self::TimeLimit(t) => write!(f, "Condition::TimeLimit({t})"),
            Self::MinPlayers(p) => write!(f, "Condition::MinPlayers({p})"),
            Self::AliveOnly => write!(f, "Condition::AliveOnly"),
            Self::DeadOnly => write!(f, "Condition::DeadOnly"),
            Self::Custom(_) => write!(f, "Condition::Custom(<fn>)"),
        }
    }
}

/// Item title representation (static string or dynamic closure).
#[derive(Clone)]
pub enum ItemTitle {
    Static(String),
    Dynamic(DynamicTitleFn),
}

impl ItemTitle {
    pub fn resolve(&self, ctx: &MenuContext) -> String {
        match self {
            Self::Static(s) => s.clone(),
            Self::Dynamic(cb) => cb(ctx),
        }
    }
}

impl std::fmt::Debug for ItemTitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(s) => write!(f, "ItemTitle::Static({s:?})"),
            Self::Dynamic(_) => write!(f, "ItemTitle::Dynamic(<fn>)"),
        }
    }
}

impl<S: Into<String>> From<S> for ItemTitle {
    fn from(s: S) -> Self {
        Self::Static(s.into())
    }
}

/// Structural kind of a menu element.
#[derive(Clone)]
pub enum ItemKind {
    /// Interactive action item assigned a numbered slot (1..8).
    Action { id: u32, action_name: String },
    /// Static informational text line without slot assignment.
    Text,
    /// Empty line spacer for visual grouping.
    Spacer,
    /// Horizontal divider (e.g. `\d-----------------------`).
    Divider(String),
}

impl std::fmt::Debug for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Action { id, action_name } => {
                write!(
                    f,
                    "ItemKind::Action {{ id: {id}, action_name: {action_name:?} }}"
                )
            }
            Self::Text => write!(f, "ItemKind::Text"),
            Self::Spacer => write!(f, "ItemKind::Spacer"),
            Self::Divider(d) => write!(f, "ItemKind::Divider({d:?})"),
        }
    }
}

/// A fully configured item within a `Menu`.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub title: ItemTitle,
    pub kind: ItemKind,
    pub conditions: Vec<Condition>,
    pub deny_policy: DenyPolicy,
    pub keep_open: bool,
    pub cooldown: Option<(std::time::Duration, AntiSpamAction)>,
}

impl MenuItem {
    /// Creates a standard interactive action item.
    pub fn new<T: Into<ItemTitle>>(title: T, id: u32) -> Self {
        Self {
            title: title.into(),
            kind: ItemKind::Action {
                id,
                action_name: String::new(),
            },
            conditions: Vec::new(),
            deny_policy: DenyPolicy::default(),
            keep_open: false,
            cooldown: None,
        }
    }

    /// Creates an action item with an explicit action name.
    pub fn with_action<T: Into<ItemTitle>, S: Into<String>>(title: T, id: u32, action: S) -> Self {
        Self {
            title: title.into(),
            kind: ItemKind::Action {
                id,
                action_name: action.into(),
            },
            conditions: Vec::new(),
            deny_policy: DenyPolicy::default(),
            keep_open: false,
            cooldown: None,
        }
    }

    /// Sets a cooldown duration for this menu item with default silent ignore behavior.
    pub fn cooldown(mut self, duration: std::time::Duration) -> Self {
        self.cooldown = Some((duration, AntiSpamAction::Ignore));
        self
    }

    /// Sets a cooldown duration and anti-spam behavioral response for this menu item.
    pub fn cooldown_with(mut self, duration: std::time::Duration, on_spam: AntiSpamAction) -> Self {
        self.cooldown = Some((duration, on_spam));
        self
    }

    /// Creates a static text element.
    pub fn text<T: Into<ItemTitle>>(title: T) -> Self {
        Self {
            title: title.into(),
            kind: ItemKind::Text,
            conditions: Vec::new(),
            deny_policy: DenyPolicy::default(),
            keep_open: false,
            cooldown: None,
        }
    }

    /// Creates an empty line spacer.
    pub fn spacer() -> Self {
        Self {
            title: ItemTitle::Static(String::new()),
            kind: ItemKind::Spacer,
            conditions: Vec::new(),
            deny_policy: DenyPolicy::default(),
            keep_open: false,
            cooldown: None,
        }
    }

    /// Creates a horizontal divider.
    pub fn divider<S: Into<String>>(divider_str: S) -> Self {
        Self {
            title: ItemTitle::Static(String::new()),
            kind: ItemKind::Divider(divider_str.into()),
            conditions: Vec::new(),
            deny_policy: DenyPolicy::default(),
            keep_open: false,
            cooldown: None,
        }
    }

    /// Attaches an access condition.
    pub fn requires(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Attaches an access condition (deprecated, use `requires`).
    #[deprecated(note = "Use .requires() instead")]
    pub fn require(self, condition: Condition) -> Self {
        self.requires(condition)
    }

    /// Configures the deny policy on condition failure.
    pub fn on_deny(mut self, policy: DenyPolicy) -> Self {
        self.deny_policy = policy;
        self
    }

    /// Shortcut: Replace title when denied.
    pub fn on_deny_replace<S: Into<String>>(self, new_title: S) -> Self {
        self.on_deny(DenyPolicy::replace(new_title))
    }
    /// Sets the item to keep the menu open after selection (re-rendering the current page).
    pub fn keep_open(mut self) -> Self {
        self.keep_open = true;
        self
    }

    /// Explicitly configures whether the item keeps the menu open after selection.
    pub fn with_keep_open(mut self, keep: bool) -> Self {
        self.keep_open = keep;
        self
    }
}

impl<S: Into<String>> From<(S, u32)> for MenuItem {
    fn from((title, id): (S, u32)) -> Self {
        Self::new(title, id)
    }
}

impl<S: Into<String>> From<(S, &'static str)> for MenuItem {
    fn from((title, action_name): (S, &'static str)) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(action_name, &mut hasher);
        let id = (std::hash::Hasher::finish(&hasher) & 0x7FFF_FFFF) as u32;
        Self::with_action(title, id, action_name)
    }
}

impl<S: Into<String>> From<(S, String)> for MenuItem {
    fn from((title, action_name): (S, String)) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&action_name, &mut hasher);
        let id = (std::hash::Hasher::finish(&hasher) & 0x7FFF_FFFF) as u32;
        Self::with_action(title, id, action_name)
    }
}

/// Navigation and exit behavior when player leaves or navigates submenus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExitBehavior {
    /// Closing menu closes all active and parent menu sessions.
    CloseAll,
    /// Closing submenu automatically pops and reopens the parent menu on the exact page it was called from.
    #[default]
    PopParent,
    /// Closing submenu returns to parent menu on a specific 1-based page index.
    /// Supports negative indices from the end (`-1` = last page, `-2` = second to last).
    PopParentPage(isize),
}

/// Visual formatting style configuration for rendered menus.
#[derive(Clone)]
pub struct MenuStyle {
    /// Header template: receives (title, current_page, max_pages).
    pub header_format: HeaderFormatFn,
    /// Active item template: receives (slot_number, item_title).
    pub item_format: ItemFormatFn,
    /// Disabled item template: receives (slot_number, item_title).
    pub disabled_item_format: ItemFormatFn,
    /// Number of action items per page (default: 7).
    pub items_per_page: usize,
    /// Back button label.
    pub back_text: String,
    /// Next button label.
    pub next_text: String,
    /// Exit button label.
    pub exit_text: String,
    /// When `true`, shifts the "Back" button from slot 8 to slot 9 on the last page (freeing slot 8 for items).
    pub dynamic_back_slot: bool,
}

impl MenuStyle {
    /// Classic GoldSrc style: `\y1.\w Title`, `\y8. Back`, `\y9. Next`, `\r0. Exit`.
    pub fn classic() -> Self {
        Self {
            header_format: Arc::new(|title, page, max_pages| {
                if max_pages > 1 {
                    format!("\\y{title}\\R\\d{page}/{max_pages}\n\n")
                } else {
                    format!("\\y{title}\n\n")
                }
            }),
            item_format: Arc::new(|slot, text| format!("\\y{slot}.\\w {text}\n")),
            disabled_item_format: Arc::new(|slot, text| format!("\\d{slot}. {text}\n")),
            items_per_page: 7,
            back_text: "\\y8. Назад\n".into(),
            next_text: "\\y9. Вперед\n".into(),
            exit_text: "\\r0. Выход\n".into(),
            dynamic_back_slot: true,
        }
    }

    /// Modern brackets style: `\r[1]\w Title`.
    pub fn brackets() -> Self {
        Self {
            header_format: Arc::new(|title, page, max_pages| {
                if max_pages > 1 {
                    format!("\\y=== {title} ===\\R\\d[{page}/{max_pages}]\n\n")
                } else {
                    format!("\\y=== {title} ===\n\n")
                }
            }),
            item_format: Arc::new(|slot, text| format!("\\r[{slot}]\\w {text}\n")),
            disabled_item_format: Arc::new(|slot, text| format!("\\d[{slot}] {text}\n")),
            items_per_page: 7,
            back_text: "\\y[8] Назад\n".into(),
            next_text: "\\y[9] Вперед\n".into(),
            exit_text: "\\r[0] Выход\n".into(),
            dynamic_back_slot: true,
        }
    }

    /// Raw uncolored style (developer provides all color tags explicitly).
    pub fn raw() -> Self {
        Self {
            header_format: Arc::new(|title, page, max_pages| {
                if max_pages > 1 {
                    format!("{title} ({page}/{max_pages})\n\n")
                } else {
                    format!("{title}\n\n")
                }
            }),
            item_format: Arc::new(|slot, text| format!("{slot}. {text}\n")),
            disabled_item_format: Arc::new(|slot, text| format!("{slot}. {text}\n")),
            items_per_page: 7,
            back_text: "8. Back\n".into(),
            next_text: "9. Next\n".into(),
            exit_text: "0. Exit\n".into(),
            dynamic_back_slot: true,
        }
    }

    /// Returns localized navigation labels (back, next, exit) for the given language code.
    pub fn localized_nav_labels(lang: &str) -> (&'static str, &'static str, &'static str) {
        match lang.to_lowercase().as_str() {
            "ru" => ("Назад", "Вперед", "Выход"),
            "es" => ("Atrás", "Siguiente", "Salir"),
            "de" => ("Zurück", "Weiter", "Beenden"),
            _ => ("Back", "Next", "Exit"),
        }
    }

    /// Adapts the style navigation texts to the specified language code.
    pub fn with_lang(mut self, lang: &str) -> Self {
        let (back, next, exit) = Self::localized_nav_labels(lang);
        let is_brackets = self.exit_text.contains('[');
        if is_brackets {
            self.back_text = format!("\\y[8] {back}\n");
            self.next_text = format!("\\y[9] {next}\n");
            self.exit_text = format!("\\r[0] {exit}\n");
        } else if self.exit_text.contains("\\r0.") || self.exit_text.contains("\\y8.") {
            self.back_text = format!("\\y8. {back}\n");
            self.next_text = format!("\\y9. {next}\n");
            self.exit_text = format!("\\r0. {exit}\n");
        } else {
            self.back_text = format!("8. {back}\n");
            self.next_text = format!("9. {next}\n");
            self.exit_text = format!("0. {exit}\n");
        }
        self
    }

    /// Sets whether the "Back" button should shift to slot 9 on the last page.
    pub fn with_dynamic_back(mut self, enabled: bool) -> Self {
        self.dynamic_back_slot = enabled;
        self
    }
}

impl Default for MenuStyle {
    fn default() -> Self {
        Self::classic()
    }
}

impl std::fmt::Debug for MenuStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuStyle")
            .field("items_per_page", &self.items_per_page)
            .field("back_text", &self.back_text)
            .field("next_text", &self.next_text)
            .field("exit_text", &self.exit_text)
            .field("dynamic_back_slot", &self.dynamic_back_slot)
            .finish()
    }
}

/// Rendering target backend for displaying the menu.
#[derive(Debug, Clone, Default)]
pub enum MenuRendererKind {
    /// Classic GoldSrc `ShowMenu` user message (`\w\y\r\d`).
    #[default]
    Text,
    /// Director HUD message (`SVC_DIRECTOR`) + invisible `ShowMenu` key interceptor.
    Dhud {
        position: crate::hud::HudCoord,
        color: crate::hud::HudColor,
        effect: crate::hud::HudEffect,
    },
}

/// Resolved interaction assigned to a single key slot (1..=10, where 10 is slot '0').
#[derive(Debug, Clone)]
pub enum SlotAction {
    /// Dispatches action item ID and name to plugin callback.
    Execute {
        id: u32,
        action_name: String,
        keep_open: bool,
    },
    /// Navigates to previous page (slot 8).
    PrevPage,
    /// Navigates to next page (slot 9).
    NextPage,
    /// Closes menu or pops parent menu (slot 10 / '0').
    Exit,
    /// Executes custom deny action feedback (sound/message).
    DenyFeedback(DenyAction),
    /// No-op action.
    Noop,
}

/// A rendered page ready to be sent over network to client.
#[derive(Debug, Clone)]
pub struct RenderedMenuPage {
    /// Formatted text buffer for `ShowMenu` or `DhudMessage`.
    pub text: String,
    /// 10-bit slot bitmask for `ShowMenu` (`(1<<0)` = 1, `(1<<9)` = 0).
    pub keys_mask: u16,
    /// Current 1-based page index.
    pub page_number: usize,
    /// Total number of pages for this player.
    pub total_pages: usize,
    /// Mapping from slot index (1..=10) to resolved action.
    pub slots: std::collections::HashMap<u8, SlotAction>,
    /// Auto-close timeout in seconds (-1 for no timeout).
    pub timeout: i32,
    /// Rendering target.
    pub renderer: MenuRendererKind,
}

/// A declarative menu definition.
#[derive(Debug, Clone)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub manual_page_breaks: Vec<usize>,
    pub style: MenuStyle,
    pub renderer: MenuRendererKind,
    pub exit_behavior: ExitBehavior,
    pub timeout_seconds: i32,
    pub required_capability: Option<String>,
    pub debounce: Option<std::time::Duration>,
}

impl Menu {
    /// Starts building a new menu with the given title.
    pub fn builder<S: Into<String>>(title: S) -> super::builder::MenuBuilder {
        super::builder::MenuBuilder::new(title)
    }

    /// Renders a specific 0-based page for the given player context.
    pub fn render_page(&self, ctx: &MenuContext, page_idx: usize) -> Option<RenderedMenuPage> {
        // 1. Filter and evaluate all items for this player
        struct EvaluatedItem {
            title: String,
            kind: ItemKind,
            is_active: bool,
            deny_action: DenyAction,
            is_forced_break: bool,
            keep_open: bool,
        }

        let mut evaluated_items = Vec::new();

        for (idx, item) in self.items.iter().enumerate() {
            let is_forced_break = self.manual_page_breaks.contains(&idx);

            // Evaluate conditions
            let mut failed_reason = None;
            for cond in &item.conditions {
                if let Err(reason) = cond.check(ctx) {
                    failed_reason = Some(reason);
                    break;
                }
            }

            if let Some(reason) = failed_reason {
                // Denied item
                match &item.deny_policy.visual {
                    VisualDeny::Hide => {
                        // Completely omit item
                        continue;
                    }
                    VisualDeny::Dimmed => {
                        let original_title = item.title.resolve(ctx);
                        evaluated_items.push(EvaluatedItem {
                            title: original_title,
                            kind: item.kind.clone(),
                            is_active: false,
                            deny_action: item.deny_policy.action.clone(),
                            is_forced_break,
                            keep_open: item.keep_open,
                        });
                    }
                    VisualDeny::Replace(replacement) => {
                        evaluated_items.push(EvaluatedItem {
                            title: replacement.clone(),
                            kind: item.kind.clone(),
                            is_active: false,
                            deny_action: item.deny_policy.action.clone(),
                            is_forced_break,
                            keep_open: item.keep_open,
                        });
                    }
                    VisualDeny::Format(formatter) => {
                        let original_title = item.title.resolve(ctx);
                        let formatted = formatter(&original_title, &reason);
                        evaluated_items.push(EvaluatedItem {
                            title: formatted,
                            kind: item.kind.clone(),
                            is_active: false,
                            deny_action: item.deny_policy.action.clone(),
                            is_forced_break,
                            keep_open: item.keep_open,
                        });
                    }
                }
            } else {
                // Active item
                let original_title = item.title.resolve(ctx);
                evaluated_items.push(EvaluatedItem {
                    title: original_title,
                    kind: item.kind.clone(),
                    is_active: true,
                    deny_action: DenyAction::Disabled,
                    is_forced_break,
                    keep_open: item.keep_open,
                });
            }
        }

        // 2. Partition into pages (considering items_per_page and forced page breaks)
        let per_page = self.style.items_per_page.clamp(1, 8);
        let mut pages: Vec<Vec<EvaluatedItem>> = Vec::new();
        let mut current_page: Vec<EvaluatedItem> = Vec::new();
        let mut action_count_on_page = 0;

        for item in evaluated_items {
            let is_action = matches!(item.kind, ItemKind::Action { .. });

            if (item.is_forced_break && !current_page.is_empty())
                || (is_action && action_count_on_page >= per_page)
            {
                pages.push(current_page);
                current_page = Vec::new();
                action_count_on_page = 0;
            }

            if is_action {
                action_count_on_page += 1;
            }
            current_page.push(item);
        }

        if !current_page.is_empty() || pages.is_empty() {
            pages.push(current_page);
        }

        let total_pages = pages.len();
        let safe_page_idx = page_idx.min(total_pages.saturating_sub(1));
        let page_items = pages.get(safe_page_idx)?;

        // 3. Format header and items
        let mut text = (self.style.header_format)(&self.title, safe_page_idx + 1, total_pages);
        let mut keys_mask: u16 = 0;
        let mut slots_map: std::collections::HashMap<u8, SlotAction> =
            std::collections::HashMap::new();
        let mut slot_counter: u8 = 1;

        for item in page_items {
            match &item.kind {
                ItemKind::Action { id, action_name } => {
                    let slot = slot_counter;
                    slot_counter += 1;

                    if item.is_active {
                        text.push_str(&(self.style.item_format)(slot as usize, &item.title));
                        keys_mask |= 1 << (slot - 1);
                        slots_map.insert(
                            slot,
                            SlotAction::Execute {
                                id: *id,
                                action_name: action_name.clone(),
                                keep_open: item.keep_open,
                            },
                        );
                    } else {
                        text.push_str(&(self.style.disabled_item_format)(
                            slot as usize,
                            &item.title,
                        ));
                        match &item.deny_action {
                            DenyAction::Disabled => {
                                // Slot excluded from mask
                            }
                            DenyAction::Noop => {
                                keys_mask |= 1 << (slot - 1);
                                slots_map.insert(slot, SlotAction::Noop);
                            }
                            DenyAction::Feedback { .. } | DenyAction::Custom(_) => {
                                keys_mask |= 1 << (slot - 1);
                                slots_map.insert(
                                    slot,
                                    SlotAction::DenyFeedback(item.deny_action.clone()),
                                );
                            }
                        }
                    }
                }
                ItemKind::Text => {
                    text.push_str(&format!("{}\n", item.title));
                }
                ItemKind::Spacer => {
                    text.push('\n');
                }
                ItemKind::Divider(divider_str) => {
                    text.push_str(&format!("{divider_str}\n"));
                }
            }
        }

        // Add padding if needed
        text.push('\n');

        // 4. Navigation Buttons (8, 9, 0)
        if total_pages > 1 {
            let has_prev = safe_page_idx > 0;
            let has_next = safe_page_idx + 1 < total_pages;

            // Slot 8 or 9: Back
            if has_prev {
                let back_slot = if self.style.dynamic_back_slot && !has_next {
                    9
                } else {
                    8
                };

                let back_text = if back_slot == 9 {
                    self.style.back_text.replace('8', "9")
                } else {
                    self.style.back_text.clone()
                };

                text.push_str(&back_text);
                keys_mask |= 1 << (back_slot - 1);
                slots_map.insert(back_slot, SlotAction::PrevPage);
            }

            // Slot 9: Next
            if has_next {
                text.push_str(&self.style.next_text);
                keys_mask |= 1 << 8; // (1<<8) = slot 9
                slots_map.insert(9, SlotAction::NextPage);
            }
        }

        // Slot 0 (key 10): Exit
        text.push_str(&self.style.exit_text);
        keys_mask |= 1 << 9; // (1<<9) = slot 0
        slots_map.insert(10, SlotAction::Exit);

        // For HUD/DHUD renderers, strip legacy ShowMenu color formatting codes (\w, \y, \r, \d, \R)
        let final_text = match &self.renderer {
            MenuRendererKind::Text => text,
            MenuRendererKind::Dhud { .. } => text
                .replace("\\y", "")
                .replace("\\r", "")
                .replace("\\w", "")
                .replace("\\d", "")
                .replace("\\R", ""),
        };

        Some(RenderedMenuPage {
            text: final_text,
            keys_mask,
            page_number: safe_page_idx + 1,
            total_pages,
            slots: slots_map,
            timeout: self.timeout_seconds,
            renderer: self.renderer.clone(),
        })
    }
}
