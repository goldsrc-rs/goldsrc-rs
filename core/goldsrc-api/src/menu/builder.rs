//! Menu definition, builder, and page layout engine.

use std::collections::HashMap;

use super::types::*;

/// Resolved interaction assigned to a single key slot (1..=10, where 10 is slot '0').
#[derive(Debug, Clone)]
pub enum SlotAction {
    /// Dispatches action item ID and name to plugin callback.
    Execute { id: u32, action_name: String },
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
    pub slots: HashMap<u8, SlotAction>,
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
}

impl Menu {
    /// Starts building a new menu with the given title.
    pub fn builder<S: Into<String>>(title: S) -> MenuBuilder {
        MenuBuilder::new(title)
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
                        });
                    }
                    VisualDeny::Replace(replacement) => {
                        evaluated_items.push(EvaluatedItem {
                            title: replacement.clone(),
                            kind: item.kind.clone(),
                            is_active: false,
                            deny_action: item.deny_policy.action.clone(),
                            is_forced_break,
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
        let mut slots_map: HashMap<u8, SlotAction> = HashMap::new();
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
            // Slot 8: Back
            if safe_page_idx > 0 {
                text.push_str(&self.style.back_text);
                keys_mask |= 1 << 7; // (1<<7) = slot 8
                slots_map.insert(8, SlotAction::PrevPage);
            }

            // Slot 9: Next
            if safe_page_idx + 1 < total_pages {
                text.push_str(&self.style.next_text);
                keys_mask |= 1 << 8; // (1<<8) = slot 9
                slots_map.insert(9, SlotAction::NextPage);
            }
        }

        // Slot 0 (key 10): Exit
        text.push_str(&self.style.exit_text);
        keys_mask |= 1 << 9; // (1<<9) = slot 0
        slots_map.insert(10, SlotAction::Exit);

        Some(RenderedMenuPage {
            text,
            keys_mask,
            page_number: safe_page_idx + 1,
            total_pages,
            slots: slots_map,
            timeout: self.timeout_seconds,
            renderer: self.renderer.clone(),
        })
    }
}

/// Fluent builder for constructing `Menu`.
#[derive(Debug, Clone)]
pub struct MenuBuilder {
    menu: Menu,
}

impl MenuBuilder {
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self {
            menu: Menu {
                title: title.into(),
                items: Vec::new(),
                manual_page_breaks: Vec::new(),
                style: MenuStyle::classic(),
                renderer: MenuRendererKind::Text,
                exit_behavior: ExitBehavior::PopParent,
                timeout_seconds: -1,
                required_capability: None,
            },
        }
    }

    /// Adds any item implementing `Into<MenuItem>` (e.g. `("Name", id)` or `MenuItem::new(...)`).
    pub fn item<I: Into<MenuItem>>(mut self, item: I) -> Self {
        self.menu.items.push(item.into());
        self
    }

    /// Adds a static text line.
    pub fn text<S: Into<String>>(mut self, text_str: S) -> Self {
        self.menu.items.push(MenuItem::text(text_str));
        self
    }

    /// Adds an empty line spacer.
    pub fn spacer(mut self) -> Self {
        self.menu.items.push(MenuItem::spacer());
        self
    }

    /// Adds a horizontal divider string.
    pub fn divider<S: Into<String>>(mut self, divider_str: S) -> Self {
        self.menu.items.push(MenuItem::divider(divider_str));
        self
    }

    /// Forces a page break at the current position.
    pub fn page_break(mut self) -> Self {
        let current_index = self.menu.items.len();
        self.menu.manual_page_breaks.push(current_index);
        self
    }

    /// Configures custom menu formatting style.
    pub fn style(mut self, style: MenuStyle) -> Self {
        self.menu.style = style;
        self
    }

    /// Sets the rendering backend (Text or Dhud).
    pub fn renderer(mut self, renderer: MenuRendererKind) -> Self {
        self.menu.renderer = renderer;
        self
    }

    /// Sets auto-close timeout in seconds (-1 for infinite).
    pub fn timeout(mut self, seconds: i32) -> Self {
        self.menu.timeout_seconds = seconds;
        self
    }

    /// Requires a specific capability to open this menu.
    pub fn require_capability<S: Into<String>>(mut self, cap: S) -> Self {
        self.menu.required_capability = Some(cap.into());
        self
    }

    /// Configures exit behavior (CloseAll or PopParent).
    pub fn exit_behavior(mut self, behavior: ExitBehavior) -> Self {
        self.menu.exit_behavior = behavior;
        self
    }

    /// Builds the configured `Menu`.
    pub fn build(self) -> Menu {
        self.menu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_basic_render() {
        let menu = Menu::builder("Test Menu")
            .item(("AK-47", 1))
            .item(("M4A1", 2))
            .item(("AWP", 3))
            .build();

        let ctx = MenuContext::new(1);
        let page = menu.render_page(&ctx, 0).expect("Page should render");

        assert_eq!(page.total_pages, 1);
        assert_eq!(page.page_number, 1);
        // Keys: 1, 2, 3, and 0 (Exit) -> (1<<0) | (1<<1) | (1<<2) | (1<<9) = 1 | 2 | 4 | 512 = 519
        assert_eq!(page.keys_mask, 519);
        assert!(page.text.contains("Test Menu"));
        assert!(page.text.contains("1."));
        assert!(page.text.contains("AK-47"));
        assert!(page.text.contains("0. Выход"));
    }

    #[test]
    fn test_menu_pagination_and_navigation() {
        let mut builder = Menu::builder("Multi Page Menu");
        for i in 1..=15 {
            builder = builder.item((format!("Weapon #{i}"), i));
        }
        let menu = builder.build();

        let ctx = MenuContext::new(1);
        let page1 = menu.render_page(&ctx, 0).expect("Page 1 renders");
        assert_eq!(page1.total_pages, 3); // 15 items / 7 per page = 3 pages
        assert_eq!(page1.page_number, 1);
        // Page 1 has items 1..7 + slot 9 (Next) + slot 0 (Exit)
        assert!(page1.slots.contains_key(&9));
        assert!(!page1.slots.contains_key(&8)); // No Back button on page 1

        let page2 = menu.render_page(&ctx, 1).expect("Page 2 renders");
        assert_eq!(page2.page_number, 2);
        // Page 2 has slot 8 (Back) and slot 9 (Next)
        assert!(page2.slots.contains_key(&8));
        assert!(page2.slots.contains_key(&9));

        let page3 = menu.render_page(&ctx, 2).expect("Page 3 renders");
        assert_eq!(page3.page_number, 3);
        // Page 3 has slot 8 (Back) and slot 0 (Exit), no Next
        assert!(page3.slots.contains_key(&8));
        assert!(!page3.slots.contains_key(&9));
    }

    #[test]
    fn test_menu_condition_replace_and_deny() {
        let menu = Menu::builder("Restricted Menu")
            .item(("Standard Knife", 1))
            .item(
                MenuItem::new("Gold AWP", 2)
                    .require(Condition::MinRound(3))
                    .on_deny_replace("[Locked: Round 3+]"),
            )
            .item(
                MenuItem::new("Admin Slay", 3)
                    .require(Condition::Capability("admin.slay".into()))
                    .on_deny(DenyPolicy::hide()),
            )
            .build();

        let mut ctx = MenuContext::new(1);
        ctx.round_number = 1; // Round 1 -> Gold AWP denied

        let page = menu.render_page(&ctx, 0).expect("Page renders");
        assert!(page.text.contains("Standard Knife"));
        assert!(page.text.contains("[Locked: Round 3+]"));
        assert!(!page.text.contains("Admin Slay")); // Hidden completely

        // Gold AWP slot (2) is disabled by default replace policy
        assert_eq!(page.keys_mask & (1 << 1), 0); // Slot 2 bit not active
    }

    #[test]
    fn test_menu_manual_page_break() {
        let menu = Menu::builder("Custom Break Menu")
            .item(("Item A", 1))
            .item(("Item B", 2))
            .page_break()
            .item(("Item C", 3))
            .build();

        let ctx = MenuContext::new(1);
        let page1 = menu.render_page(&ctx, 0).expect("Page 1 renders");
        assert_eq!(page1.total_pages, 2);
        assert!(page1.text.contains("Item A"));
        assert!(page1.text.contains("Item B"));
        assert!(!page1.text.contains("Item C"));

        let page2 = menu.render_page(&ctx, 1).expect("Page 2 renders");
        assert!(page2.text.contains("Item C"));
    }
}
