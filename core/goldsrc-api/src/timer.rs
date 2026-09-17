//! Discrete tick and continuous duration task scheduling abstractions.

use crate::client::PlayerSessionToken;
use core::fmt;
use core::ops::{Add, Deref, Sub};
use core::time::Duration;

/// Discrete physics or engine frame tick count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ticks(pub u64);

impl Ticks {
    /// Zero ticks (immediate).
    pub const ZERO: Self = Self(0);

    /// Creates a new discrete tick count.
    #[inline(always)]
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns the raw tick number.
    #[inline(always)]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for Ticks {
    #[inline(always)]
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl From<Ticks> for u64 {
    #[inline(always)]
    fn from(ticks: Ticks) -> Self {
        ticks.0
    }
}

impl Deref for Ticks {
    type Target = u64;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Ticks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ticks", self.0)
    }
}

impl Add for Ticks {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for Ticks {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

/// Task scheduling delay, either in discrete engine ticks or wall-clock duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScheduleDelay {
    /// Delay by discrete engine frames/ticks.
    Ticks(u64),
    /// Delay by real/engine duration.
    Duration(Duration),
}

impl fmt::Display for ScheduleDelay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ticks(t) => write!(f, "{t} ticks"),
            Self::Duration(d) => write!(f, "{:.2}s", d.as_secs_f32()),
        }
    }
}

/// Helper trait to convert various units into [`ScheduleDelay`].
pub trait IntoScheduleDelay {
    /// Converts `self` into a [`ScheduleDelay`].
    fn into_schedule_delay(self) -> ScheduleDelay;
}

impl IntoScheduleDelay for ScheduleDelay {
    #[inline(always)]
    fn into_schedule_delay(self) -> ScheduleDelay {
        self
    }
}

impl IntoScheduleDelay for Ticks {
    #[inline(always)]
    fn into_schedule_delay(self) -> ScheduleDelay {
        ScheduleDelay::Ticks(self.0)
    }
}

impl IntoScheduleDelay for Duration {
    #[inline(always)]
    fn into_schedule_delay(self) -> ScheduleDelay {
        ScheduleDelay::Duration(self)
    }
}

/// Unique identifier for an enqueued timer or repeating task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimerId(pub u64);

impl fmt::Display for TimerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TimerId({})", self.0)
    }
}

/// Return outcome of a recurring timer callback indicating whether to repeat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerAction {
    /// Keep running and schedule the next invocation.
    #[default]
    Continue,
    /// Cancel and do not invoke again.
    Stop,
}

/// Repetition mode of a scheduled task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerMode {
    /// One-shot execution.
    Once,
    /// Periodic repetition with the given interval.
    Recurring(ScheduleDelay),
}

/// Execution constraint binding for a scheduled timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimerBound {
    /// Global task, executes independently of connected players.
    #[default]
    Global,
    /// Bound to a specific player session. If the session terminates or is recycled, the timer is aborted.
    Session(PlayerSessionToken),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticks_operations() {
        let t1 = Ticks::new(10);
        let t2 = Ticks::from(5u64);
        assert_eq!(*t1, 10);
        assert_eq!(u64::from(t2), 5);
        assert_eq!(t1 + t2, Ticks::new(15));
        assert_eq!(t1 - t2, Ticks::new(5));
        assert_eq!(t2 - t1, Ticks::ZERO);
    }

    #[test]
    fn test_schedule_delay_conversions() {
        let d1 = Ticks(42).into_schedule_delay();
        assert_eq!(d1, ScheduleDelay::Ticks(42));

        let dur = Duration::from_millis(500);
        let d2 = dur.into_schedule_delay();
        assert_eq!(d2, ScheduleDelay::Duration(dur));

        let d3 = ScheduleDelay::Ticks(100).into_schedule_delay();
        assert_eq!(d3, ScheduleDelay::Ticks(100));
    }
}
