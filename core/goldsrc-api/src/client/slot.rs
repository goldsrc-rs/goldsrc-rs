//! Thread-safe slot and identifier handles for off-thread communication.

use crate::Entity;
use crate::client::{Client, ClientExt, Player};

/// A lightweight, copyable player slot index (1..=32) that is safe to pass across threads.
///
/// Unlike [`Player`] and [`Client`], which hold engine handles and are strictly bound
/// to the GoldSrc engine main thread (`!Send` and `!Sync`), [`PlayerSlot`] is a pure POD
/// identifier that implements [`Send`] and [`Sync`].
///
/// To perform engine operations or mutate properties, background tasks pass this slot back
/// to the main thread (e.g. via task dispatch), where it can be securely resolved via
/// [`PlayerSlot::resolve`] or [`PlayerSlot::resolve_client`].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PlayerSlot(pub i32);

impl PlayerSlot {
    /// Creates a new `PlayerSlot` from a 1-based client index.
    #[inline(always)]
    pub const fn new(index: i32) -> Self {
        Self(index)
    }

    /// Returns the raw 1-based slot index (1..=32).
    #[inline(always)]
    pub const fn index(self) -> i32 {
        self.0
    }

    /// Returns `true` if this slot index is within the valid GoldSrc client range (1..=32).
    #[inline(always)]
    pub const fn is_valid_range(self) -> bool {
        self.0 >= 1 && self.0 <= 32
    }

    /// Resolves this slot into a live [`Player`] handle on the main thread.
    ///
    /// Returns `None` if the slot is out of bounds, disconnected, or is an HLTV proxy.
    #[inline]
    pub fn resolve(self) -> Option<Player> {
        if !self.is_valid_range() {
            return None;
        }
        let player = Player::new(self.0);
        if player.is_valid() && !player.is_hltv() {
            Some(player)
        } else {
            None
        }
    }

    /// Resolves this slot into a live [`Client`] handle on the main thread (including HLTV).
    #[inline]
    pub fn resolve_client(self) -> Option<Client> {
        if !self.is_valid_range() {
            return None;
        }
        let client = Client::new(self.0);
        if client.is_valid() {
            Some(client)
        } else {
            None
        }
    }
}

impl From<i32> for PlayerSlot {
    #[inline(always)]
    fn from(index: i32) -> Self {
        Self(index)
    }
}

impl From<PlayerSlot> for i32 {
    #[inline(always)]
    fn from(slot: PlayerSlot) -> Self {
        slot.0
    }
}

impl From<Player> for PlayerSlot {
    #[inline(always)]
    fn from(player: Player) -> Self {
        Self(player.index())
    }
}

impl From<&Player> for PlayerSlot {
    #[inline(always)]
    fn from(player: &Player) -> Self {
        Self(player.index())
    }
}

impl From<Client> for PlayerSlot {
    #[inline(always)]
    fn from(client: Client) -> Self {
        Self(client.index())
    }
}

impl From<&Client> for PlayerSlot {
    #[inline(always)]
    fn from(client: &Client) -> Self {
        Self(client.index())
    }
}

/// A lightweight, copyable entity identifier that is safe to pass across threads.
///
/// Unlike [`Entity`], which holds engine handles and is strictly bound to the GoldSrc
/// main thread (`!Send` and `!Sync`), [`EntityId`] is a pure POD identifier.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct EntityId(pub i32);

impl EntityId {
    /// Creates a new `EntityId` from a 0-based entity index (0 = world).
    #[inline(always)]
    pub const fn new(index: i32) -> Self {
        Self(index)
    }

    /// Returns the raw entity index.
    #[inline(always)]
    pub const fn index(self) -> i32 {
        self.0
    }

    /// Resolves this ID into a live [`Entity`] handle on the main thread.
    ///
    /// Returns `None` if the entity is invalid or freed.
    #[inline]
    pub fn resolve(self) -> Option<Entity> {
        let ent = Entity::new(self.0);
        if ent.is_valid() { Some(ent) } else { None }
    }
}

impl From<i32> for EntityId {
    #[inline(always)]
    fn from(index: i32) -> Self {
        Self(index)
    }
}

impl From<EntityId> for i32 {
    #[inline(always)]
    fn from(id: EntityId) -> Self {
        id.0
    }
}

impl From<Entity> for EntityId {
    #[inline(always)]
    fn from(entity: Entity) -> Self {
        Self(entity.index())
    }
}

impl From<&Entity> for EntityId {
    #[inline(always)]
    fn from(entity: &Entity) -> Self {
        Self(entity.index())
    }
}

impl From<Client> for EntityId {
    #[inline(always)]
    fn from(client: Client) -> Self {
        Self(client.index())
    }
}

impl From<Player> for EntityId {
    #[inline(always)]
    fn from(player: Player) -> Self {
        Self(player.index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn test_slots_are_send_and_sync() {
        assert_send::<PlayerSlot>();
        assert_sync::<PlayerSlot>();
        assert_send::<EntityId>();
        assert_sync::<EntityId>();
    }

    #[test]
    fn test_player_slot_bounds() {
        assert!(!PlayerSlot::new(0).is_valid_range());
        assert!(PlayerSlot::new(1).is_valid_range());
        assert!(PlayerSlot::new(32).is_valid_range());
        assert!(!PlayerSlot::new(33).is_valid_range());
        assert_eq!(PlayerSlot::new(7).index(), 7);
    }

    #[test]
    fn test_entity_id() {
        let id = EntityId::new(128);
        assert_eq!(id.index(), 128);
        assert_eq!(i32::from(id), 128);
    }

    #[test]
    fn test_player_slot_from_player() {
        let player = Player::new(5);
        let slot: PlayerSlot = player.slot();
        assert_eq!(slot.index(), 5);
        assert_eq!(PlayerSlot::from(player), PlayerSlot(5));
        assert_eq!(PlayerSlot::from(&player), PlayerSlot(5));
    }

    #[test]
    fn test_entity_id_from_entity() {
        let entity = Entity::new(42);
        let id: EntityId = entity.id();
        assert_eq!(id.index(), 42);
        assert_eq!(EntityId::from(entity), EntityId(42));
        assert_eq!(EntityId::from(&entity), EntityId(42));
    }
}
