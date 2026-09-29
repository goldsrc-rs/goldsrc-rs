//! Decoupled View Renderers for the GoldSrc.rs MVC Menu System.
//!
//! Provides the [`MenuRenderer`] trait and concrete implementations for
//! rendering menu pages across diverse display channels:
//! - [`ClassicMenuRenderer`]: standard Half-Life `ShowMenu` (`\w`, `\y`, `\r`, `\d`)
//! - [`DhudMenuRenderer`]: high-fidelity Director HUD overlay with ghost slot key trapping
//! - [`ChatMenuRenderer`]: compact chat-based menu for spectators or minimal UI
//! - [`MotdMenuRenderer`]: rich interactive HTML/CSS dialogs (rules, leaderboards, stats)
//! - [`TerminalTuiRenderer`]: server-side console TUI / admin dashboard

use super::types::{Menu, MenuContext, MenuRendererKind, RenderedMenuPage, SlotAction};
use crate::hud::{HudColor, HudCoord, HudEffect};
use std::fmt::Write as _;

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

/// Compact chat-based menu renderer (for spectators, low-res clients, or notification menus).
#[derive(Debug, Clone, Copy, Default)]
pub struct ChatMenuRenderer;

impl MenuRenderer for ChatMenuRenderer {
    fn render_page(&self, menu: &Menu, page: usize, ctx: &MenuContext) -> Option<RenderedMenuPage> {
        let mut rendered = menu.render_page(ctx, page)?;
        // Convert menu lines into colored chat lines (^3 Team, ^4 Green, ^1 Normal)
        let mut chat_lines = String::with_capacity(rendered.text.len() + 32);
        for line in rendered.text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let clean = strip_goldsrc_colors(line);
            if clean.starts_with(|c: char| c.is_ascii_digit()) {
                let _ = writeln!(chat_lines, "^4[Menu]^1 {clean}");
            } else {
                let _ = writeln!(chat_lines, "^3{clean}");
            }
        }
        rendered.text = chat_lines;
        rendered.renderer = MenuRendererKind::Chat;
        Some(rendered)
    }

    fn kind(&self) -> MenuRendererKind {
        MenuRendererKind::Chat
    }
}

/// Interactive MOTD (Message of the Day) HTML/CSS dialog renderer.
#[derive(Debug, Clone)]
pub struct MotdMenuRenderer {
    pub dialog_title: String,
    pub dark_theme: bool,
}

impl Default for MotdMenuRenderer {
    fn default() -> Self {
        Self {
            dialog_title: "GoldSrc.rs Interactive Menu".into(),
            dark_theme: true,
        }
    }
}

impl MenuRenderer for MotdMenuRenderer {
    fn render_page(&self, menu: &Menu, page: usize, ctx: &MenuContext) -> Option<RenderedMenuPage> {
        let rendered = menu.render_page(ctx, page)?;
        let bg_color = if self.dark_theme {
            "#121212"
        } else {
            "#f5f5f5"
        };
        let text_color = if self.dark_theme {
            "#e0e0e0"
        } else {
            "#212121"
        };
        let card_bg = if self.dark_theme {
            "#1e1e1e"
        } else {
            "#ffffff"
        };
        let accent = "#4caf50";

        let mut html = String::with_capacity(2048);
        let _ = write!(
            html,
            r#"<!DOCTYPE html><html><head><meta charset="utf-8">
<style>
body {{ background-color: {bg_color}; color: {text_color}; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; padding: 20px; }}
.card {{ background-color: {card_bg}; border-radius: 8px; padding: 16px; box-shadow: 0 4px 6px rgba(0,0,0,0.3); max-width: 600px; margin: 0 auto; }}
h2 {{ color: {accent}; margin-top: 0; border-bottom: 2px solid {accent}; padding-bottom: 8px; }}
.item {{ padding: 10px; margin: 6px 0; border-radius: 4px; background: rgba(255,255,255,0.05); display: flex; align-items: center; }}
.slot {{ font-weight: bold; color: {accent}; margin-right: 12px; width: 24px; }}
.footer {{ margin-top: 16px; font-size: 0.85em; opacity: 0.7; text-align: right; }}
</style></head><body><div class="card"><h2>{} (Page {}/{})</h2>"#,
            menu.title, rendered.page_number, rendered.total_pages
        );

        for (slot, action) in &rendered.slots {
            let action_name = match action {
                SlotAction::Execute { action_name, .. } => action_name.as_str(),
                SlotAction::NextPage => "Next Page",
                SlotAction::PrevPage => "Previous Page",
                SlotAction::Exit => "Exit",
                _ => continue,
            };
            let _ = write!(
                html,
                r#"<div class="item"><span class="slot">{}</span><span>{}</span></div>"#,
                slot, action_name
            );
        }

        let _ = write!(
            html,
            r#"<div class="footer">GoldSrc.rs MVC Web UI</div></div></body></html>"#
        );

        Some(RenderedMenuPage {
            text: html,
            keys_mask: rendered.keys_mask,
            page_number: rendered.page_number,
            total_pages: rendered.total_pages,
            slots: rendered.slots,
            timeout: rendered.timeout,
            renderer: MenuRendererKind::Motd {
                title: self.dialog_title.clone(),
                dark_theme: self.dark_theme,
            },
        })
    }

    fn kind(&self) -> MenuRendererKind {
        MenuRendererKind::Motd {
            title: self.dialog_title.clone(),
            dark_theme: self.dark_theme,
        }
    }
}

/// Server console / TUI menu renderer with ASCII/Unicode box-drawing characters.
#[derive(Debug, Clone, Copy, Default)]
pub struct TerminalTuiRenderer;

impl MenuRenderer for TerminalTuiRenderer {
    fn render_page(&self, menu: &Menu, page: usize, ctx: &MenuContext) -> Option<RenderedMenuPage> {
        let rendered = menu.render_page(ctx, page)?;
        let width = 48;
        let mut box_buf = String::with_capacity(1024);

        // Header border
        box_buf.push('┌');
        for _ in 0..width {
            box_buf.push('─');
        }
        box_buf.push_str("┐\n");

        // Title line
        let title_line = format!(
            " {} ({}/{})",
            menu.title, rendered.page_number, rendered.total_pages
        );
        let title_padded = format!("{:width$}", title_line, width = width);
        let _ = writeln!(box_buf, "│{title_padded}│");

        // Divider
        box_buf.push('├');
        for _ in 0..width {
            box_buf.push('─');
        }
        box_buf.push_str("┤\n");

        // Items
        for line in rendered.text.lines() {
            let clean = strip_goldsrc_colors(line);
            if clean.trim().is_empty() {
                continue;
            }
            let truncated = if clean.chars().count() > width - 2 {
                clean.chars().take(width - 5).collect::<String>() + "..."
            } else {
                clean
            };
            let padded = format!(" {:width$}", truncated, width = width - 1);
            let _ = writeln!(box_buf, "│{padded}│");
        }

        // Bottom border
        box_buf.push('└');
        for _ in 0..width {
            box_buf.push('─');
        }
        box_buf.push_str("┘\n");

        Some(RenderedMenuPage {
            text: box_buf,
            keys_mask: rendered.keys_mask,
            page_number: rendered.page_number,
            total_pages: rendered.total_pages,
            slots: rendered.slots,
            timeout: rendered.timeout,
            renderer: MenuRendererKind::TerminalTui,
        })
    }

    fn kind(&self) -> MenuRendererKind {
        MenuRendererKind::TerminalTui
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

        let chat = ChatMenuRenderer;
        let rendered_chat = chat.render_page(&menu, 1, &ctx).unwrap();
        assert!(rendered_chat.text.contains("^4[Menu]^1"));

        let motd = MotdMenuRenderer::default();
        let rendered_motd = motd.render_page(&menu, 1, &ctx).unwrap();
        assert!(rendered_motd.text.contains("<!DOCTYPE html>"));

        let tui = TerminalTuiRenderer;
        let rendered_tui = tui.render_page(&menu, 1, &ctx).unwrap();
        assert!(rendered_tui.text.contains('┌'));
    }
}
