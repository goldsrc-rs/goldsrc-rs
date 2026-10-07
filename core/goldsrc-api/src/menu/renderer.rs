//! Decoupled View Renderers for the GoldSrc.rs MVC Menu System.
//!
//! Provides the [`MenuRenderer`] trait and concrete implementations for
//! rendering menu pages across diverse display channels:
//! - [`ClassicMenuRenderer`]: standard Half-Life `ShowMenu` (`\w`, `\y`, `\r`, `\d`)
//! - [`DhudMenuRenderer`]: high-fidelity Director HUD overlay with ghost slot key trapping
//! - [`ChatMenuRenderer`]: compact chat-based menu for spectators or minimal UI
//! - [`MotdMenuRenderer`]: rich interactive HTML/CSS dialogs (rules, leaderboards, stats)
//! - [`TerminalTuiRenderer`]: server-side console TUI / admin dashboard

use super::types::{Menu, MenuContext, MenuRendererKind, RenderedMenuPage};
use crate::hud::{HudColor, HudCoord, HudEffect};

/// Trait implemented by view renderers in the MVC Menu System.
pub trait MenuRenderer: Send + Sync {
    /// Renders a specific page of a menu for the given context.
    fn render_page(&self, menu: &Menu, page: usize, ctx: &MenuContext) -> Option<RenderedMenuPage>;

    /// Returns the renderer kind discriminant.
    fn kind(&self) -> MenuRendererKind;
}

/// Classic Half-Life `ShowMenu` view renderer (`\w` white, `\y` yellow, `\r` red, `\d` dimmed).
#[derive(Debug, Clone, Copy, Default)]
pub struct ClassicMenuRenderer;

impl MenuRenderer for ClassicMenuRenderer {
    fn render_page(&self, menu: &Menu, page: usize, ctx: &MenuContext) -> Option<RenderedMenuPage> {
        menu.render_page(ctx, page)
    }

    fn kind(&self) -> MenuRendererKind {
        MenuRendererKind::Text
    }
}

/// High-fidelity Director HUD (`SVC_DIRECTOR`) view renderer with ghost slot key trapping.
#[derive(Debug, Clone)]
pub struct DhudMenuRenderer {
    pub position: HudCoord,
    pub color: HudColor,
    pub effect: HudEffect,
}

impl Default for DhudMenuRenderer {
    fn default() -> Self {
        Self {
            position: HudCoord { x: 0.05, y: 0.25 },
            color: HudColor {
                r: 100,
                g: 200,
                b: 255,
                a: 255,
            },
            effect: HudEffect::default(),
        }
    }
}

impl MenuRenderer for DhudMenuRenderer {
    fn render_page(&self, menu: &Menu, page: usize, ctx: &MenuContext) -> Option<RenderedMenuPage> {
        let mut rendered = menu.render_page(ctx, page)?;
        // Strip raw GoldSrc color formatting codes (\w, \y, \r, \d) for clean DHUD font rendering
        let clean_text = strip_goldsrc_colors(&rendered.text);
        rendered.text = clean_text;
        rendered.renderer = MenuRendererKind::Dhud {
            position: self.position,
            color: self.color,
            effect: self.effect,
        };
        // Ghost slot trap: Ensure keys_mask has all interactive slots enabled
        // so client engine captures 1..=10 without switching weapons.
        if rendered.keys_mask == 0 {
            rendered.keys_mask = 0x3FF;
        }
        Some(rendered)
    }

    fn kind(&self) -> MenuRendererKind {
        MenuRendererKind::Dhud {
            position: self.position,
            color: self.color,
            effect: self.effect,
        }
    }
}

/// Helper to strip GoldSrc escape color sequences (`\w`, `\y`, `\r`, `\d`, `\R`).
pub fn strip_goldsrc_colors(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && matches!(chars.peek(), Some('w' | 'y' | 'r' | 'd' | 'R')) {
            chars.next();
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu::Menu;

    #[test]
    fn test_strip_goldsrc_colors() {
        assert_eq!(
            strip_goldsrc_colors(r"\y1. \wBuy \rAWP\d (Unavailable)"),
            "1. Buy AWP (Unavailable)"
        );
    }

    #[test]
    fn test_renderers_produce_output() {
        let menu = Menu::builder("Test Menu")
            .action("Option 1", 1)
            .action("Option 2", 2)
            .build();
        let ctx = MenuContext::new(1);

        let classic = ClassicMenuRenderer;
        assert!(classic.render_page(&menu, 1, &ctx).is_some());

        let dhud = DhudMenuRenderer::default();
        let rendered_dhud = dhud.render_page(&menu, 1, &ctx).unwrap();
        assert!(!rendered_dhud.text.contains(r"\y"));
    }
}
