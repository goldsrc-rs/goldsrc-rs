use crate::hud::HudColor;

/// Flags controlling ScreenFade behavior (matching HLSDK `FFADE_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FadeFlags(pub u16);

impl FadeFlags {
    /// Fade in from color to normal vision.
    pub const IN: Self = Self(0x0000);
    /// Fade out from normal vision to solid color.
    pub const OUT: Self = Self(0x0001);
    /// Hold the fade color indefinitely until reset.
    pub const HOLD: Self = Self(0x0002);
    /// Modulate color with current screen content.
    pub const MODULATE: Self = Self(0x0004);
    /// Stay active through map/round transitions.
    pub const STAY_OUT: Self = Self(0x0008);

    /// Combines two flag sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Checks if a flag is contained.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// A declarative screen fade effect descriptor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenFade {
    /// Duration of the fade ramp in seconds.
    pub duration: f32,
    /// Hold time in seconds at peak intensity.
    pub hold_time: f32,
    /// Behavior flags (`FFADE_IN`, `FFADE_OUT`, `FFADE_HOLD`, etc.).
    pub flags: FadeFlags,
    /// Color and alpha intensity of the fade.
    pub color: HudColor,
}

impl Default for ScreenFade {
    fn default() -> Self {
        Self {
            duration: 1.0,
            hold_time: 0.5,
            flags: FadeFlags::OUT,
            color: HudColor::new(255, 0, 0, 180),
        }
    }
}

impl ScreenFade {
    /// Creates a builder for a screen fade effect.
    pub fn builder() -> ScreenFadeBuilder {
        ScreenFadeBuilder::default()
    }

    /// Predefined flashbang blind effect (White, long hold).
    pub fn flashbang(duration: f32, hold: f32) -> Self {
        Self {
            duration,
            hold_time: hold,
            flags: FadeFlags::OUT.union(FadeFlags::HOLD),
            color: HudColor::new(255, 255, 255, 255),
        }
    }

    /// Predefined damage flash effect (Red, fast fade-in).
    pub fn damage_flash() -> Self {
        Self {
            duration: 0.2,
            hold_time: 0.1,
            flags: FadeFlags::OUT,
            color: HudColor::new(255, 0, 0, 120),
        }
    }
}

/// Fluent builder for constructing `ScreenFade`.
#[derive(Debug, Clone, Default)]
pub struct ScreenFadeBuilder {
    fade: ScreenFade,
}

impl ScreenFadeBuilder {
    /// Sets the ramp duration in seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.fade.duration = duration;
        self
    }

    /// Sets the hold time in seconds.
    pub fn hold_time(mut self, hold: f32) -> Self {
        self.fade.hold_time = hold;
        self
    }

    /// Sets behavior flags.
    pub fn flags(mut self, flags: FadeFlags) -> Self {
        self.fade.flags = flags;
        self
    }

    /// Sets RGBA color.
    pub fn color(mut self, color: HudColor) -> Self {
        self.fade.color = color;
        self
    }

    /// Sets RGB color with custom alpha.
    pub fn rgba(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.fade.color = HudColor::new(r, g, b, a);
        self
    }

    /// Builds the `ScreenFade` configuration.
    pub fn build(self) -> ScreenFade {
        self.fade
    }
}

/// A declarative screen shake / earthquake descriptor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenShake {
    /// Amplitude of the shake in screen units.
    pub amplitude: f32,
    /// Duration of the shake in seconds.
    pub duration: f32,
    /// Frequency / speed of oscillations.
    pub frequency: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            amplitude: 4.0,
            duration: 1.0,
            frequency: 50.0,
        }
    }
}

impl ScreenShake {
    /// Creates a builder for a screen shake effect.
    pub fn builder() -> ScreenShakeBuilder {
        ScreenShakeBuilder::default()
    }

    /// Predefined explosion tremor effect.
    pub fn explosion() -> Self {
        Self {
            amplitude: 8.0,
            duration: 1.5,
            frequency: 100.0,
        }
    }
}

/// Fluent builder for constructing `ScreenShake`.
#[derive(Debug, Clone, Default)]
pub struct ScreenShakeBuilder {
    shake: ScreenShake,
}

impl ScreenShakeBuilder {
    /// Sets shake amplitude.
    pub fn amplitude(mut self, amp: f32) -> Self {
        self.shake.amplitude = amp;
        self
    }

    /// Sets shake duration in seconds.
    pub fn duration(mut self, duration: f32) -> Self {
        self.shake.duration = duration;
        self
    }

    /// Sets oscillation frequency.
    pub fn frequency(mut self, freq: f32) -> Self {
        self.shake.frequency = freq;
        self
    }

    /// Builds the `ScreenShake` configuration.
    pub fn build(self) -> ScreenShake {
        self.shake
    }
}
