//! Extensible Widget Component Model for the GoldSrc.rs MVC Menu System.
//!
//! Provides the [`MenuComponent`] trait and built-in interactive widgets:
//! - [`Checkbox`]: interactive toggle switch with visual `[X]` / `[ ]` indicators.
//! - [`Slider`]: numeric range adjustment with visual progress bar `[====|====] 50%`.
//! - [`TextInput`]: client text query utilizing Half-Life's native `messagemode` command.

use super::types::MenuContext;

/// Trait implemented by custom interactive widgets in declarative menus.
pub trait MenuComponent: Send + Sync {
    /// Formats the widget's label and value into a single displayable line of text.
    fn render(&self, ctx: &MenuContext) -> String;

    /// Unique action identifier or callback name when interacted with.
    fn action_name(&self) -> &str;

    /// Invoked when the user triggers the component (clicks slot or presses Select).
    fn on_interact(&mut self, ctx: &MenuContext);
}

/// Interactive Checkbox component (`[X]` enabled, `[ ]` disabled).
#[derive(Debug, Clone)]
pub struct Checkbox {
    pub label: String,
    pub checked: bool,
    pub action: String,
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool, action: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked,
            action: action.into(),
        }
    }

    pub fn toggle(&mut self) -> bool {
        self.checked = !self.checked;
        self.checked
    }
}

impl MenuComponent for Checkbox {
    fn render(&self, _ctx: &MenuContext) -> String {
        let indicator = if self.checked { "[X]" } else { "[ ]" };
        format!("{} {}", indicator, self.label)
    }

    fn action_name(&self) -> &str {
        &self.action
    }

    fn on_interact(&mut self, _ctx: &MenuContext) {
        self.toggle();
    }
}

/// Numeric slider component with a visual ASCII bar.
#[derive(Debug, Clone)]
pub struct Slider {
    pub label: String,
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub current: i32,
    pub action: String,
}

impl Slider {
    pub fn new(
        label: impl Into<String>,
        min: i32,
        max: i32,
        step: i32,
        initial: i32,
        action: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            min,
            max,
            step,
            current: initial.clamp(min, max),
            action: action.into(),
        }
    }

    pub fn increment(&mut self) -> i32 {
        self.current = (self.current + self.step).min(self.max);
        self.current
    }

    pub fn decrement(&mut self) -> i32 {
        self.current = (self.current - self.step).max(self.min);
        self.current
    }

    pub fn progress_ratio(&self) -> f32 {
        if self.max <= self.min {
            0.0
        } else {
            (self.current - self.min) as f32 / (self.max - self.min) as f32
        }
    }

    pub fn render_bar(&self, segments: usize) -> String {
        let ratio = self.progress_ratio();
        let filled = ((segments as f32) * ratio).round() as usize;
        let mut bar = String::with_capacity(segments + 2);
        bar.push('[');
        for i in 0..segments {
            if i < filled {
                bar.push('=');
            } else if i == filled {
                bar.push('|');
            } else {
                bar.push('-');
            }
        }
        bar.push(']');
        bar
    }
}

impl MenuComponent for Slider {
    fn render(&self, _ctx: &MenuContext) -> String {
        let bar = self.render_bar(10);
        format!("{}: {} ({})", self.label, bar, self.current)
    }

    fn action_name(&self) -> &str {
        &self.action
    }

    fn on_interact(&mut self, _ctx: &MenuContext) {
        if self.current >= self.max {
            self.current = self.min;
        } else {
            self.increment();
        }
    }
}

/// Text input component that transitions client into `messagemode` text entry.
#[derive(Debug, Clone)]
pub struct TextInput {
    pub label: String,
    pub prompt: String,
    pub action: String,
}

impl TextInput {
    pub fn new(
        label: impl Into<String>,
        prompt: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            prompt: prompt.into(),
            action: action.into(),
        }
    }

    /// Formats the client command needed to prompt `messagemode`.
    pub fn command_string(&self) -> String {
        format!("messagemode {}\n", self.action)
    }
}

impl MenuComponent for TextInput {
    fn render(&self, _ctx: &MenuContext) -> String {
        format!("{}: [ {} ]", self.label, self.prompt)
    }

    fn action_name(&self) -> &str {
        &self.action
    }

    fn on_interact(&mut self, _ctx: &MenuContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_toggle() {
        let mut cb = Checkbox::new("Auto-bhop", false, "toggle_bhop");
        let ctx = MenuContext::new(1);
        assert_eq!(cb.render(&ctx), "[ ] Auto-bhop");
        cb.on_interact(&ctx);
        assert!(cb.checked);
        assert_eq!(cb.render(&ctx), "[X] Auto-bhop");
    }

    #[test]
    fn test_slider_bounds() {
        let mut sl = Slider::new("Volume", 0, 100, 10, 50, "set_volume");
        let ctx = MenuContext::new(1);
        assert_eq!(sl.current, 50);
        assert!(sl.render(&ctx).contains("50"));
        sl.increment();
        assert_eq!(sl.current, 60);
        for _ in 0..10 {
            sl.increment();
        }
        assert_eq!(sl.current, 100);
        for _ in 0..15 {
            sl.decrement();
        }
        assert_eq!(sl.current, 0);
    }

    #[test]
    fn test_text_input_command() {
        let ti = TextInput::new("Reason", "Enter ban reason", "admin_ban_reason");
        assert_eq!(ti.command_string(), "messagemode admin_ban_reason\n");
    }
}
