//! HUD and DHUD screen message types and builders.

/// RGBA color representation for screen HUD/DHUD messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HudColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl HudColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const YELLOW: Self = Self::rgb(255, 215, 0);
    pub const RED: Self = Self::rgb(255, 64, 64);
    pub const GREEN: Self = Self::rgb(64, 255, 64);
    pub const BLUE: Self = Self::rgb(64, 128, 255);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const ORANGE: Self = Self::rgb(255, 140, 0);
    pub const GOLD: Self = Self::rgb(255, 200, 50);
}

impl Default for HudColor {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Normalized 2D screen coordinate.
/// `-1.0` indicates centered on that axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HudCoord {
    pub x: f32,
    pub y: f32,
}

impl HudCoord {
    pub const CENTER: Self = Self { x: -1.0, y: -1.0 };
    pub const TOP_CENTER: Self = Self { x: -1.0, y: 0.15 };
    pub const BOTTOM_CENTER: Self = Self { x: -1.0, y: 0.8 };
    pub const MENU_DEFAULT: Self = Self { x: 0.05, y: 0.3 };

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Default for HudCoord {
    fn default() -> Self {
        Self::CENTER
    }
}

/// Animation effects for screen HUD / DHUD messages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HudEffect {
    /// Standard fade-in, hold, and fade-out.
    FadeInOut {
        fade_in: f32,
        fade_out: f32,
        hold_time: f32,
    },
    /// Flickering / blinking message with secondary color.
    Flicker { fx_time: f32, hold_time: f32 },
    /// Character-by-character typewriter effect.
    Typewriter {
        char_time: f32,
        fade_out: f32,
        hold_time: f32,
    },
}

impl Default for HudEffect {
    fn default() -> Self {
        Self::FadeInOut {
            fade_in: 0.1,
            fade_out: 0.2,
            hold_time: 4.0,
        }
    }
}

/// Type of HUD screen message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HudKind {
    /// Classic 4-channel HUD message (`SVC_TEMPENTITY` / `TE_TEXTMESSAGE`).
    Classic { channel: u8 },
    /// Director HUD message (`SVC_DIRECTOR` / `DrcCmd`), large font, no 4-channel slot limit.
    #[default]
    Dhud,
}

/// A declarative screen HUD / DHUD message descriptor.
#[derive(Debug, Clone)]
pub struct HudMessage {
    pub text: String,
    pub kind: HudKind,
    pub color: HudColor,
    pub color2: HudColor,
    pub position: HudCoord,
    pub effect: HudEffect,
}

impl HudMessage {
    /// Creates a new HUD message with default styling (DHUD, White, Centered).
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            kind: HudKind::Dhud,
            color: HudColor::WHITE,
            color2: HudColor::WHITE,
            position: HudCoord::CENTER,
            effect: HudEffect::default(),
        }
    }

    /// Creates a builder for a HUD message.
    pub fn builder<S: Into<String>>(text: S) -> HudMessageBuilder {
        HudMessageBuilder::new(text)
    }
}

/// Fluent builder for constructing `HudMessage`.
#[derive(Debug, Clone)]
pub struct HudMessageBuilder {
    msg: HudMessage,
}

impl HudMessageBuilder {
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            msg: HudMessage::new(text),
        }
    }

    /// Sets the HUD rendering mode to DHUD (Director HUD, default).
    pub fn dhud(mut self) -> Self {
        self.msg.kind = HudKind::Dhud;
        self
    }

    /// Sets the HUD rendering mode to Classic with a specific channel (1..=4).
    pub fn classic(mut self, channel: u8) -> Self {
        self.msg.kind = HudKind::Classic {
            channel: channel.clamp(1, 4),
        };
        self
    }

    /// Sets primary RGBA color.
    pub fn color(mut self, color: HudColor) -> Self {
        self.msg.color = color;
        self
    }

    /// Sets primary RGB color.
    pub fn rgb(mut self, r: u8, g: u8, b: u8) -> Self {
        self.msg.color = HudColor::rgb(r, g, b);
        self
    }

    /// Sets secondary RGBA color (for flicker effects).
    pub fn color2(mut self, color: HudColor) -> Self {
        self.msg.color2 = color;
        self
    }

    /// Sets screen coordinates.
    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.msg.position = HudCoord::new(x, y);
        self
    }

    /// Sets animation effect.
    pub fn effect(mut self, effect: HudEffect) -> Self {
        self.msg.effect = effect;
        self
    }

    /// Sets simple fade-in / fade-out timing.
    pub fn timing(mut self, fade_in: f32, fade_out: f32, hold_time: f32) -> Self {
        self.msg.effect = HudEffect::FadeInOut {
            fade_in,
            fade_out,
            hold_time,
        };
        self
    }

    /// Builds the configured `HudMessage`.
    pub fn build(self) -> HudMessage {
        self.msg
    }
}
