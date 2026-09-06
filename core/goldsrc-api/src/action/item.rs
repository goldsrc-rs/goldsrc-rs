//! Item and weapon delivery action.

use crate::action::Action;
use crate::client::Player;

/// Spawns an item or weapon entity by classname and delivers it to the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GiveItem {
    /// Entity classname to give (e.g. `"weapon_ak47"`, `"item_assaultsuit"`).
    pub item: String,
}

impl GiveItem {
    /// Creates a new item delivery action.
    pub fn new(item: impl Into<String>) -> Self {
        Self { item: item.into() }
    }
}

impl Action<Player> for GiveItem {
    type Output = Option<i32>;

    #[inline(always)]
    fn execute(self, player: &Player) -> Self::Output {
        if !player.is_valid() {
            return None;
        }

        #[cfg(target_arch = "wasm32")]
        {
            use crate::bindings::goldsrc::engine::api as host;
            let ent = host::host_create_named_entity(&self.item)?;
            let o = host::host_entity_origin(player.index);
            host::host_entity_set_origin(
                ent,
                crate::bindings::goldsrc::engine::api::Vector3 {
                    x: o.x,
                    y: o.y,
                    z: o.z,
                },
            );
            host::host_dispatch_spawn(ent);
            host::host_dispatch_touch(ent, player.index);
            Some(ent)
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (player, self);
            None
        }
    }
}
