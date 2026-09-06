//! Audio playback action.

use crate::Entity;
use crate::action::Action;
use crate::client::Player;

/// Plays an audio sample effect directly to the player's client or from an entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaySound {
    /// Relative sound filepath within the game directory (e.g. `"buttons/button10.wav"`).
    pub sample: String,
}

impl PlaySound {
    /// Creates a new sound playback action.
    pub fn new(sample: impl Into<String>) -> Self {
        Self {
            sample: sample.into(),
        }
    }
}

impl Action<Entity> for PlaySound {
    type Output = ();

    #[inline(always)]
    fn execute(self, entity: &Entity) -> Self::Output {
        if !entity.is_valid() {
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            crate::bindings::goldsrc::engine::api::host_emit_sound(
                entity.index,
                0, // CHAN_AUTO
                &self.sample,
                1.0, // VOL_NORM
                1.0, // ATTN_NORM
                0,
                100, // PITCH_NORM
            );
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (entity, self.sample);
        }
    }
}

impl Action<Player> for PlaySound {
    type Output = ();

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        self.execute(&**player)
    }
}
