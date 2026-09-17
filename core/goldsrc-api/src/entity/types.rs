//! Entity rendering, collision, and physics mode enumerations.

/// Visual rendering mode (`rendermode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum RenderMode {
    /// Normal opaque rendering.
    #[default]
    Normal = 0,
    /// Color-modulated rendering.
    TransColor = 1,
    /// Texture transparency (e.g. index 255 transparent blue in sprites).
    TransTexture = 2,
    /// Glowing light-source or sprite glow.
    Glow = 3,
    /// Solid alpha testing.
    TransSolid = 4,
    /// Additive blending (e.g. fire, beams, energy sprites).
    TransAdd = 5,
}

impl RenderMode {
    /// Converts from a raw engine integer representation.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::TransColor,
            2 => Self::TransTexture,
            3 => Self::Glow,
            4 => Self::TransSolid,
            5 => Self::TransAdd,
            _ => Self::Normal,
        }
    }

    /// Returns the integer representation expected by the GoldSrc engine.
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<i32> for RenderMode {
    fn from(val: i32) -> Self {
        Self::from_raw(val)
    }
}

impl From<RenderMode> for i32 {
    fn from(val: RenderMode) -> Self {
        val.as_raw()
    }
}

/// Visual rendering special effects (`renderfx`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum RenderFx {
    /// No special effect.
    #[default]
    None = 0,
    PulseSlow = 1,
    PulseFast = 2,
    PulseSlowWide = 3,
    PulseFastWide = 4,
    FadeSlow = 5,
    FadeFast = 6,
    SolidSlow = 7,
    SolidFast = 8,
    StrobeSlow = 9,
    StrobeFast = 10,
    StrobeFaster = 11,
    FlickerSlow = 12,
    FlickerFast = 13,
    NoDissipation = 14,
    Distort = 15,
    Hologram = 16,
    DeadPlayer = 17,
    Explode = 18,
    GlowShell = 19,
    ClampMinScale = 20,
}

impl RenderFx {
    /// Converts from a raw engine integer representation.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::PulseSlow,
            2 => Self::PulseFast,
            3 => Self::PulseSlowWide,
            4 => Self::PulseFastWide,
            5 => Self::FadeSlow,
            6 => Self::FadeFast,
            7 => Self::SolidSlow,
            8 => Self::SolidFast,
            9 => Self::StrobeSlow,
            10 => Self::StrobeFast,
            11 => Self::StrobeFaster,
            12 => Self::FlickerSlow,
            13 => Self::FlickerFast,
            14 => Self::NoDissipation,
            15 => Self::Distort,
            16 => Self::Hologram,
            17 => Self::DeadPlayer,
            18 => Self::Explode,
            19 => Self::GlowShell,
            20 => Self::ClampMinScale,
            _ => Self::None,
        }
    }

    /// Returns the integer representation expected by the GoldSrc engine.
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<i32> for RenderFx {
    fn from(val: i32) -> Self {
        Self::from_raw(val)
    }
}

impl From<RenderFx> for i32 {
    fn from(val: RenderFx) -> Self {
        val.as_raw()
    }
}

/// Physical solidity type (`solid`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum SolidType {
    /// Non-solid, passes through everything without collision.
    #[default]
    Not = 0,
    /// Trigger entity (fires Touch callbacks without physical collision response).
    Trigger = 1,
    /// Axial bounding box collision.
    Bbox = 2,
    /// Sliding bounding box (players and monsters).
    Slidebox = 3,
    /// Brush geometry collision (BSP models).
    Bsp = 4,
}

impl SolidType {
    /// Converts from a raw engine integer representation.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::Trigger,
            2 => Self::Bbox,
            3 => Self::Slidebox,
            4 => Self::Bsp,
            _ => Self::Not,
        }
    }

    /// Returns the integer representation expected by the GoldSrc engine.
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<i32> for SolidType {
    fn from(val: i32) -> Self {
        Self::from_raw(val)
    }
}

impl From<SolidType> for i32 {
    fn from(val: SolidType) -> Self {
        val.as_raw()
    }
}

/// Physical movement type (`movetype`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum MoveType {
    /// Stationary entity.
    #[default]
    None = 0,
    /// Standard player physics with gravity, step climbing, friction.
    Walk = 3,
    /// Monster physics.
    Step = 4,
    /// Flying entity without gravity.
    Fly = 5,
    /// Ballistic trajectory with gravity.
    Toss = 6,
    /// Moving brush entity (doors, trains, platforms).
    Push = 7,
    /// Ghost movement through walls and geometry.
    Noclip = 8,
    /// High-velocity missile.
    FlyMissile = 9,
    /// Bouncing physics with partial restitution.
    Bounce = 10,
    /// Bouncing missile.
    BounceMissile = 11,
    /// Follows target entity.
    Follow = 12,
    /// Push step movement.
    PushStep = 13,
}

impl MoveType {
    /// Converts from a raw engine integer representation.
    pub const fn from_raw(raw: i32) -> Self {
        match raw {
            3 => Self::Walk,
            4 => Self::Step,
            5 => Self::Fly,
            6 => Self::Toss,
            7 => Self::Push,
            8 => Self::Noclip,
            9 => Self::FlyMissile,
            10 => Self::Bounce,
            11 => Self::BounceMissile,
            12 => Self::Follow,
            13 => Self::PushStep,
            _ => Self::None,
        }
    }

    /// Returns the integer representation expected by the GoldSrc engine.
    pub const fn as_raw(self) -> i32 {
        self as i32
    }
}

impl From<i32> for MoveType {
    fn from(val: i32) -> Self {
        Self::from_raw(val)
    }
}

impl From<MoveType> for i32 {
    fn from(val: MoveType) -> Self {
        val.as_raw()
    }
}
