//! Typestate zero-cost wrappers and state-verified guards.

use crate::Vector3;
use crate::client::{ClientExt, EntityExt, Player, PlayerExt};
use std::ops::{Deref, DerefMut};

/// Typestate extractor guaranteeing that the wrapped player/entity is currently alive (`health > 0`).
#[derive(Debug, Clone, PartialEq)]
pub struct Alive<T>(pub T);

impl<T> Deref for Alive<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Alive<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: EntityExt> EntityExt for Alive<T> {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.0.origin()
    }
    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.0.set_origin(pos);
    }
    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.0.velocity()
    }
    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.0.set_velocity(vel);
    }
    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.0.angles()
    }
    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.0.set_angles(angles);
    }
    #[inline(always)]
    fn health(&self) -> f32 {
        self.0.health()
    }
    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.0.set_health(health);
    }
    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.0.classname()
    }
    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.0.is_alive()
    }
    #[inline(always)]
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl<T: ClientExt> ClientExt for Alive<T> {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.0.client_index()
    }
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.0.name()
    }
    #[inline(always)]
    fn lang(&self) -> String {
        self.0.lang()
    }
    #[inline(always)]
    fn client_kind(&self) -> crate::client::ClientKind {
        self.0.client_kind()
    }
    #[inline(always)]
    fn is_bot(&self) -> bool {
        self.0.is_bot()
    }
    #[inline(always)]
    fn is_hltv(&self) -> bool {
        self.0.is_hltv()
    }
    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.0.print_console(msg);
    }
    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.0.print_notify(msg);
    }
}

impl<T: PlayerExt> PlayerExt for Alive<T> {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.0.armorvalue()
    }
    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.0.set_armorvalue(armor);
    }
    #[inline(always)]
    fn team(&self) -> crate::client::Team {
        self.0.team()
    }
    #[inline(always)]
    fn life_state(&self) -> crate::client::LifeState {
        self.0.life_state()
    }
    #[inline(always)]
    fn print(&self, target: crate::client::PrintTarget, msg: impl Into<String>) {
        self.0.print(target, msg);
    }
    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.0.print_chat(msg);
    }
    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.0.print_center(msg);
    }
    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.0.print_color(msg);
    }
    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.0.play_sound(sample);
    }
    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.0.open_menu(menu);
    }
    #[inline(always)]
    fn show_menu(&self, menu: &crate::menu::Menu) {
        self.0.show_menu(menu);
    }
    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.0.show_raw_menu(keys_mask, timeout, text);
    }
    #[inline(always)]
    fn close_menu(&self) {
        self.0.close_menu();
    }
    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.0.send_hud(msg);
    }
    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.0.give_item(item)
    }
    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.0.has_capability(name)
    }
    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.0.grant_capability(name)
    }
    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.0.revoke_capability(name)
    }
}

/// Typestate extractor guaranteeing that the wrapped player/entity is currently dead (`health <= 0`).
#[derive(Debug, Clone, PartialEq)]
pub struct Dead<T>(pub T);

impl<T> Deref for Dead<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Dead<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: EntityExt> EntityExt for Dead<T> {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.0.origin()
    }
    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.0.set_origin(pos);
    }
    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.0.velocity()
    }
    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.0.set_velocity(vel);
    }
    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.0.angles()
    }
    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.0.set_angles(angles);
    }
    #[inline(always)]
    fn health(&self) -> f32 {
        self.0.health()
    }
    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.0.set_health(health);
    }
    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.0.classname()
    }
    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.0.is_alive()
    }
    #[inline(always)]
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl<T: ClientExt> ClientExt for Dead<T> {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.0.client_index()
    }
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.0.name()
    }
    #[inline(always)]
    fn lang(&self) -> String {
        self.0.lang()
    }
    #[inline(always)]
    fn client_kind(&self) -> crate::client::ClientKind {
        self.0.client_kind()
    }
    #[inline(always)]
    fn is_bot(&self) -> bool {
        self.0.is_bot()
    }
    #[inline(always)]
    fn is_hltv(&self) -> bool {
        self.0.is_hltv()
    }
    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.0.print_console(msg);
    }
    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.0.print_notify(msg);
    }
}

impl<T: PlayerExt> PlayerExt for Dead<T> {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.0.armorvalue()
    }
    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.0.set_armorvalue(armor);
    }
    #[inline(always)]
    fn team(&self) -> crate::client::Team {
        self.0.team()
    }
    #[inline(always)]
    fn life_state(&self) -> crate::client::LifeState {
        self.0.life_state()
    }
    #[inline(always)]
    fn print(&self, target: crate::client::PrintTarget, msg: impl Into<String>) {
        self.0.print(target, msg);
    }
    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.0.print_chat(msg);
    }
    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.0.print_center(msg);
    }
    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.0.print_color(msg);
    }
    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.0.play_sound(sample);
    }
    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.0.open_menu(menu);
    }
    #[inline(always)]
    fn show_menu(&self, menu: &crate::menu::Menu) {
        self.0.show_menu(menu);
    }
    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.0.show_raw_menu(keys_mask, timeout, text);
    }
    #[inline(always)]
    fn close_menu(&self) {
        self.0.close_menu();
    }
    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.0.send_hud(msg);
    }
    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.0.give_item(item)
    }
    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.0.has_capability(name)
    }
    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.0.grant_capability(name)
    }
    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.0.revoke_capability(name)
    }
}

/// Typestate extractor guaranteeing that the caller is on the Spectator team.
#[derive(Debug, Clone, PartialEq)]
pub struct Spectator(pub Player);

impl Deref for Spectator {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Spectator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EntityExt for Spectator {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.0.origin()
    }
    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.0.set_origin(pos);
    }
    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.0.velocity()
    }
    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.0.set_velocity(vel);
    }
    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.0.angles()
    }
    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.0.set_angles(angles);
    }
    #[inline(always)]
    fn health(&self) -> f32 {
        self.0.health()
    }
    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.0.set_health(health);
    }
    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.0.classname()
    }
    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.0.is_alive()
    }
    #[inline(always)]
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl ClientExt for Spectator {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.0.client_index()
    }
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.0.name()
    }
    #[inline(always)]
    fn lang(&self) -> String {
        self.0.lang()
    }
    #[inline(always)]
    fn client_kind(&self) -> crate::client::ClientKind {
        self.0.client_kind()
    }
    #[inline(always)]
    fn is_bot(&self) -> bool {
        self.0.is_bot()
    }
    #[inline(always)]
    fn is_hltv(&self) -> bool {
        self.0.is_hltv()
    }
    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.0.print_console(msg);
    }
    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.0.print_notify(msg);
    }
}

impl PlayerExt for Spectator {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.0.armorvalue()
    }
    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.0.set_armorvalue(armor);
    }
    #[inline(always)]
    fn team(&self) -> crate::client::Team {
        self.0.team()
    }
    #[inline(always)]
    fn life_state(&self) -> crate::client::LifeState {
        self.0.life_state()
    }
    #[inline(always)]
    fn print(&self, target: crate::client::PrintTarget, msg: impl Into<String>) {
        self.0.print(target, msg);
    }
    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.0.print_chat(msg);
    }
    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.0.print_center(msg);
    }
    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.0.print_color(msg);
    }
    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.0.play_sound(sample);
    }
    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.0.open_menu(menu);
    }
    #[inline(always)]
    fn show_menu(&self, menu: &crate::menu::Menu) {
        self.0.show_menu(menu);
    }
    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.0.show_raw_menu(keys_mask, timeout, text);
    }
    #[inline(always)]
    fn close_menu(&self) {
        self.0.close_menu();
    }
    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.0.send_hud(msg);
    }
    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.0.give_item(item)
    }
    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.0.has_capability(name)
    }
    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.0.grant_capability(name)
    }
    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.0.revoke_capability(name)
    }
}

/// Typestate extractor representing an AI Bot client (`FL_FAKECLIENT`).
#[derive(Debug, Clone, PartialEq)]
pub struct Bot(pub Player);

impl Deref for Bot {
    type Target = Player;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EntityExt for Bot {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.0.origin()
    }
    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.0.set_origin(pos);
    }
    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.0.velocity()
    }
    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.0.set_velocity(vel);
    }
    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.0.angles()
    }
    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.0.set_angles(angles);
    }
    #[inline(always)]
    fn health(&self) -> f32 {
        self.0.health()
    }
    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.0.set_health(health);
    }
    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.0.classname()
    }
    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.0.is_alive()
    }
    #[inline(always)]
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl ClientExt for Bot {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.0.client_index()
    }
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.0.name()
    }
    #[inline(always)]
    fn lang(&self) -> String {
        self.0.lang()
    }
    #[inline(always)]
    fn client_kind(&self) -> crate::client::ClientKind {
        crate::client::ClientKind::Bot
    }
    #[inline(always)]
    fn is_bot(&self) -> bool {
        true
    }
    #[inline(always)]
    fn is_hltv(&self) -> bool {
        false
    }
    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.0.print_console(msg);
    }
    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.0.print_notify(msg);
    }
}

impl PlayerExt for Bot {
    #[inline(always)]
    fn armorvalue(&self) -> f32 {
        self.0.armorvalue()
    }
    #[inline(always)]
    fn set_armorvalue(&mut self, armor: f32) {
        self.0.set_armorvalue(armor);
    }
    #[inline(always)]
    fn team(&self) -> crate::client::Team {
        self.0.team()
    }
    #[inline(always)]
    fn life_state(&self) -> crate::client::LifeState {
        self.0.life_state()
    }
    #[inline(always)]
    fn print(&self, target: crate::client::PrintTarget, msg: impl Into<String>) {
        self.0.print(target, msg);
    }
    #[inline(always)]
    fn print_chat(&self, msg: impl Into<String>) {
        self.0.print_chat(msg);
    }
    #[inline(always)]
    fn print_center(&self, msg: impl Into<String>) {
        self.0.print_center(msg);
    }
    #[inline(always)]
    fn print_color(&self, msg: impl Into<String>) {
        self.0.print_color(msg);
    }
    #[inline(always)]
    fn play_sound(&self, sample: impl Into<String>) {
        self.0.play_sound(sample);
    }
    #[inline(always)]
    fn open_menu(&self, menu: &crate::menu::Menu) {
        self.0.open_menu(menu);
    }
    #[inline(always)]
    fn show_menu(&self, menu: &crate::menu::Menu) {
        self.0.show_menu(menu);
    }
    #[inline(always)]
    fn show_raw_menu(&self, keys_mask: i32, timeout: i32, text: &str) {
        self.0.show_raw_menu(keys_mask, timeout, text);
    }
    #[inline(always)]
    fn close_menu(&self) {
        self.0.close_menu();
    }
    #[inline(always)]
    fn send_hud(&self, msg: &crate::hud::HudMessage) {
        self.0.send_hud(msg);
    }
    #[inline(always)]
    fn give_item(&self, item: impl Into<String>) -> Option<i32> {
        self.0.give_item(item)
    }
    #[inline(always)]
    fn has_capability(&self, name: &str) -> bool {
        self.0.has_capability(name)
    }
    #[inline(always)]
    fn grant_capability(&self, name: impl Into<String>) -> bool {
        self.0.grant_capability(name)
    }
    #[inline(always)]
    fn revoke_capability(&self, name: impl Into<String>) -> bool {
        self.0.revoke_capability(name)
    }
}

/// Typestate extractor representing an HLTV relay proxy client (`FL_PROXY`).
///
/// Implements [`EntityExt`] and [`ClientExt`], but intentionally does NOT implement [`PlayerExt`].
/// This ensures at compile time that operations like giving items, opening interactive menus,
/// or combatant actions cannot be performed on an HLTV spectator proxy.
#[derive(Debug, Clone, PartialEq)]
pub struct Hltv(pub Player);

impl Deref for Hltv {
    type Target = crate::Entity;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hltv {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<crate::Entity> for Hltv {
    #[inline(always)]
    fn as_ref(&self) -> &crate::Entity {
        &self.0
    }
}

impl EntityExt for Hltv {
    #[inline(always)]
    fn origin(&self) -> Vector3 {
        self.0.origin()
    }
    #[inline(always)]
    fn set_origin(&mut self, pos: Vector3) {
        self.0.set_origin(pos);
    }
    #[inline(always)]
    fn velocity(&self) -> Vector3 {
        self.0.velocity()
    }
    #[inline(always)]
    fn set_velocity(&mut self, vel: Vector3) {
        self.0.set_velocity(vel);
    }
    #[inline(always)]
    fn angles(&self) -> Vector3 {
        self.0.angles()
    }
    #[inline(always)]
    fn set_angles(&mut self, angles: Vector3) {
        self.0.set_angles(angles);
    }
    #[inline(always)]
    fn health(&self) -> f32 {
        self.0.health()
    }
    #[inline(always)]
    fn set_health(&mut self, health: f32) {
        self.0.set_health(health);
    }
    #[inline(always)]
    fn classname(&self) -> Option<String> {
        self.0.classname()
    }
    #[inline(always)]
    fn is_alive(&self) -> bool {
        self.0.is_alive()
    }
    #[inline(always)]
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
}

impl ClientExt for Hltv {
    #[inline(always)]
    fn client_index(&self) -> i32 {
        self.0.client_index()
    }
    #[inline(always)]
    fn name(&self) -> Option<String> {
        self.0.name()
    }
    #[inline(always)]
    fn lang(&self) -> String {
        self.0.lang()
    }
    #[inline(always)]
    fn client_kind(&self) -> crate::client::ClientKind {
        crate::client::ClientKind::Hltv
    }
    #[inline(always)]
    fn is_bot(&self) -> bool {
        false
    }
    #[inline(always)]
    fn is_hltv(&self) -> bool {
        true
    }
    #[inline(always)]
    fn print_console(&self, msg: impl Into<String>) {
        self.0.print_console(msg);
    }
    #[inline(always)]
    fn print_notify(&self, msg: impl Into<String>) {
        self.0.print_notify(msg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Entity;

    fn assert_is_entity<T: EntityExt>(_target: &T) {}
    fn assert_is_client<T: ClientExt>(_target: &T) {}
    fn assert_is_player<T: PlayerExt>(_target: &T) {}

    #[test]
    fn test_entity_ext_hierarchy() {
        let ent = Entity::new(0);
        assert_is_entity(&ent);
        assert_eq!(ent.origin(), Vector3::new(0.0, 0.0, 0.0));
        assert_eq!(ent.health(), 0.0);
        assert!(!ent.is_alive());
    }

    #[test]
    fn test_player_implements_all_tiers() {
        let player = Player::new(1);
        assert_is_entity(&player);
        assert_is_client(&player);
        assert_is_player(&player);
        assert_eq!(player.client_index(), 1);
        assert_eq!(player.origin(), Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_bot_implements_all_tiers() {
        let bot = Bot(Player::new(2));
        assert_is_entity(&bot);
        assert_is_client(&bot);
        assert_is_player(&bot);
        assert_eq!(bot.client_index(), 2);
        assert!(bot.is_bot());
        assert!(!bot.is_hltv());
    }

    #[test]
    fn test_hltv_implements_client_and_entity_but_not_player() {
        let hltv = Hltv(Player::new(32));
        assert_is_entity(&hltv);
        assert_is_client(&hltv);
        assert_eq!(hltv.client_index(), 32);
        assert!(hltv.is_hltv());
        assert!(!hltv.is_bot());
        assert_eq!(hltv.client_kind(), crate::client::ClientKind::Hltv);
        assert_eq!(hltv.origin(), Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_alive_and_dead_guards_preserve_tiers() {
        let alive_player = Alive(Player::new(3));
        assert_is_entity(&alive_player);
        assert_is_client(&alive_player);
        assert_is_player(&alive_player);

        let dead_player = Dead(Player::new(4));
        assert_is_entity(&dead_player);
        assert_is_client(&dead_player);
        assert_is_player(&dead_player);

        let alive_hltv = Alive(Hltv(Player::new(5)));
        assert_is_entity(&alive_hltv);
        assert_is_client(&alive_hltv);
    }
}
