//! Fluent builder for constructing `Menu`.

use super::types::*;

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

    /// Appends a dedicated page configured via a nested builder closure.
    pub fn page<F>(mut self, f: F) -> Self
    where
        F: FnOnce(MenuPageBuilder) -> MenuPageBuilder,
    {
        if !self.menu.items.is_empty() {
            let current_index = self.menu.items.len();
            self.menu.manual_page_breaks.push(current_index);
        }
        let pb = f(MenuPageBuilder::new());
        self.menu.items.extend(pb.items);
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

    /// Sets the target language code for automatic navigation buttons localization.
    pub fn lang<S: AsRef<str>>(mut self, lang: S) -> Self {
        self.menu.style = self.menu.style.with_lang(lang.as_ref());
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

    /// Configures exit behavior (CloseAll, PopParent, etc.).
    pub fn exit_behavior(mut self, behavior: ExitBehavior) -> Self {
        self.menu.exit_behavior = behavior;
        self
    }

    /// Sets exit behavior to return to parent menu on a specific page number (e.g. `1` = page 1, `-1` = last page).
    pub fn exit_to_parent_page(mut self, page: isize) -> Self {
        self.menu.exit_behavior = ExitBehavior::PopParentPage(page);
        self
    }

    /// Builds the configured `Menu`.
    pub fn build(self) -> Menu {
        self.menu
    }
}

/// Builder for constructing items within a specific explicit menu page.
#[derive(Debug, Default, Clone)]
pub struct MenuPageBuilder {
    pub(crate) items: Vec<MenuItem>,
}

impl MenuPageBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds any item implementing `Into<MenuItem>` (e.g. `("Name", id)` or `MenuItem::new(...)`).
    pub fn item<I: Into<MenuItem>>(mut self, item: I) -> Self {
        self.items.push(item.into());
        self
    }

    /// Adds a static text line.
    pub fn text<S: Into<String>>(mut self, text_str: S) -> Self {
        self.items.push(MenuItem::text(text_str));
        self
    }

    /// Adds an empty line spacer.
    pub fn spacer(mut self) -> Self {
        self.items.push(MenuItem::spacer());
        self
    }

    /// Adds a horizontal divider string.
    pub fn divider<S: Into<String>>(mut self, divider_str: S) -> Self {
        self.items.push(MenuItem::divider(divider_str));
        self
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
        // Page 3 has dynamic slot 9 (Back on last page) and slot 10 (Exit)
        assert!(page3.slots.contains_key(&9));
        assert!(page3.slots.contains_key(&10));
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
