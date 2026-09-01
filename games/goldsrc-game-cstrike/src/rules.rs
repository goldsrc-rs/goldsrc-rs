//! Counter-Strike 1.6 game rules, round phases, and win conditions.

/// Current CS 1.6 round state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundState {
    /// Freeze period before round start.
    FreezePeriod,
    /// Active gameplay in progress.
    Active,
    /// Round concluded (waiting for reset/respawn).
    RoundEnded,
}

/// Round termination reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundEndReason {
    TargetBombed,
    VipEscaped,
    VipAssassinated,
    TerroristsEscaped,
    CtPreventEscape,
    TerroristsStopped,
    BombDefused,
    CtWin,
    TerroristWin,
    RoundDraw,
    AllHostagesRescued,
    TargetSaved,
    HostagesNotRescued,
    TerroristsNotEscaped,
}
