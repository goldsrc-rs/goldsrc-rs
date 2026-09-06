//! C++ Virtual Table (VTable) and Entity Lifecycle Hooks.
//!
//! Provides dynamic virtual function interception for `CBaseEntity`, `CBasePlayer`,
//! and weapon/item entities (e.g. `TakeDamage`, `Killed`, `TraceAttack`, `Spawn`, `ResetMaxSpeed`).

use crate::hooks::types::{HookResult, HookTiming};
use goldsrc_api::gamedata::VTableFunc;
use goldsrc_api::modifiers::{CommutativeModifier, TypedBlackboard};
use std::collections::HashMap;
use std::sync::RwLock;

/// Payload for `TakeDamage` virtual function call.
#[derive(Debug, Clone, PartialEq)]
pub struct TakeDamageContext {
    /// Victim entity index.
    pub victim: i32,
    /// Inflictor entity index (e.g. grenade, rocket, or weapon holder).
    pub inflictor: i32,
    /// Attacker entity index (e.g. player who fired).
    pub attacker: i32,
    /// Amount of damage being dealt (can be mutated in Pre-hook or resolved via `modifiers`).
    pub damage: f32,
    /// Damage type bits (`DMG_GENERIC`, `DMG_BULLET`, `DMG_BLAST`, etc.).
    pub bits_damage_type: i32,
    /// Algebraic, order-independent damage modifier pipeline.
    pub modifiers: CommutativeModifier,
    /// Context blackboard for inter-plugin auxiliary metadata.
    pub blackboard: TypedBlackboard,
}

impl TakeDamageContext {
    /// Creates a new `TakeDamageContext` initializing the commutative modifier pipeline with base damage.
    pub fn new(
        victim: i32,
        inflictor: i32,
        attacker: i32,
        damage: f32,
        bits_damage_type: i32,
    ) -> Self {
        Self {
            victim,
            inflictor,
            attacker,
            damage,
            bits_damage_type,
            modifiers: CommutativeModifier::new(damage),
            blackboard: TypedBlackboard::new(),
        }
    }

    /// Synchronizes `self.damage` with the computed commutative modifier result.
    pub fn sync_damage(&mut self) -> f32 {
        if !self.modifiers.flat_bonuses.is_empty()
            || !self.modifiers.multipliers.is_empty()
            || !self.modifiers.reductions.is_empty()
            || self.modifiers.is_blocked
        {
            self.damage = self.modifiers.compute();
        } else {
            self.modifiers.base = self.damage;
        }
        self.damage
    }
}

/// Payload for `Killed` virtual function call.
#[derive(Debug, Clone, PartialEq)]
pub struct KilledContext {
    /// Victim entity index.
    pub victim: i32,
    /// Attacker entity index.
    pub attacker: i32,
    /// Gib behavior mode (GIB_NORMAL, GIB_NEVER, GIB_ALWAYS).
    pub gib_mode: i32,
}

/// Callback type for TakeDamage entity hooks.
pub type TakeDamageHook =
    Box<dyn Fn(&mut TakeDamageContext, HookTiming) -> HookResult<i32> + Send + Sync + 'static>;
/// Callback type for Killed entity hooks.
pub type KilledHook =
    Box<dyn Fn(&KilledContext, HookTiming) -> HookResult<()> + Send + Sync + 'static>;
/// Callback type for generic single-entity hooks (Spawn, ResetMaxSpeed, etc.).
pub type EntityHook = Box<dyn Fn(i32, HookTiming) -> HookResult<()> + Send + Sync + 'static>;

/// Central runtime registry for dynamic entity vtable hooks.
#[derive(Default)]
pub struct EntityHookRegistry {
    take_damage_hooks: Vec<(Option<String>, HookTiming, TakeDamageHook)>,
    killed_hooks: Vec<(Option<String>, HookTiming, KilledHook)>,
    generic_hooks: HashMap<VTableFunc, Vec<(Option<String>, HookTiming, EntityHook)>>,
}

impl EntityHookRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a TakeDamage hook for a specific classname (or all if `None`).
    pub fn register_take_damage<F>(
        &mut self,
        classname: Option<&str>,
        timing: HookTiming,
        callback: F,
    ) where
        F: Fn(&mut TakeDamageContext, HookTiming) -> HookResult<i32> + Send + Sync + 'static,
    {
        self.take_damage_hooks
            .push((classname.map(|s| s.to_string()), timing, Box::new(callback)));
    }

    /// Registers a Killed hook for a specific classname (or all if `None`).
    pub fn register_killed<F>(&mut self, classname: Option<&str>, timing: HookTiming, callback: F)
    where
        F: Fn(&KilledContext, HookTiming) -> HookResult<()> + Send + Sync + 'static,
    {
        self.killed_hooks
            .push((classname.map(|s| s.to_string()), timing, Box::new(callback)));
    }

    /// Registers a generic hook for a specific VTableFunc.
    pub fn register_hook<F>(
        &mut self,
        func: VTableFunc,
        classname: Option<&str>,
        timing: HookTiming,
        callback: F,
    ) where
        F: Fn(i32, HookTiming) -> HookResult<()> + Send + Sync + 'static,
    {
        self.generic_hooks.entry(func).or_default().push((
            classname.map(|s| s.to_string()),
            timing,
            Box::new(callback),
        ));
    }

    /// Dispatches TakeDamage pre/post hooks.
    pub fn dispatch_take_damage(
        &self,
        ctx: &mut TakeDamageContext,
        timing: HookTiming,
    ) -> HookResult<i32> {
        let mut final_result = HookResult::Ignored;
        for (_, hook_timing, callback) in &self.take_damage_hooks {
            if *hook_timing == timing {
                let res = callback(ctx, timing);
                ctx.sync_damage();
                if res.is_superceded() {
                    return res;
                }
                if res == HookResult::Handled {
                    final_result = HookResult::Handled;
                }
            }
        }

        // Emit WASM event: "entity_take_damage_pre" or "entity_take_damage_post"
        // Payload layout: [victim: i32, inflictor: i32, attacker: i32, damage: f32, bits_damage_type: i32] (20 bytes)
        let event_name = match timing {
            HookTiming::Pre => "entity_take_damage_pre",
            HookTiming::Post => "entity_take_damage_post",
        };
        let mut payload = [0u8; 20];
        payload[0..4].copy_from_slice(&ctx.victim.to_le_bytes());
        payload[4..8].copy_from_slice(&ctx.inflictor.to_le_bytes());
        payload[8..12].copy_from_slice(&ctx.attacker.to_le_bytes());
        payload[12..16].copy_from_slice(&ctx.damage.to_le_bytes());
        payload[16..20].copy_from_slice(&ctx.bits_damage_type.to_le_bytes());
        crate::hooks::dispatcher::emit_event(event_name, &payload);

        final_result
    }

    /// Dispatches Killed pre/post hooks.
    pub fn dispatch_killed(&self, ctx: &KilledContext, timing: HookTiming) -> HookResult<()> {
        let mut final_result = HookResult::Ignored;
        for (_, hook_timing, callback) in &self.killed_hooks {
            if *hook_timing == timing {
                let res = callback(ctx, timing);
                if res.is_superceded() {
                    return res;
                }
                if res == HookResult::Handled {
                    final_result = HookResult::Handled;
                }
            }
        }

        // Emit WASM event: "entity_killed_pre" or "entity_killed_post"
        // Payload layout: [victim: i32, attacker: i32, gib_mode: i32] (12 bytes)
        let event_name = match timing {
            HookTiming::Pre => "entity_killed_pre",
            HookTiming::Post => "entity_killed_post",
        };
        let mut payload = [0u8; 12];
        payload[0..4].copy_from_slice(&ctx.victim.to_le_bytes());
        payload[4..8].copy_from_slice(&ctx.attacker.to_le_bytes());
        payload[8..12].copy_from_slice(&ctx.gib_mode.to_le_bytes());
        crate::hooks::dispatcher::emit_event(event_name, &payload);

        final_result
    }

    /// Dispatches generic entity hook.
    pub fn dispatch_generic(
        &self,
        func: VTableFunc,
        entity_idx: i32,
        timing: HookTiming,
    ) -> HookResult<()> {
        let mut final_result = HookResult::Ignored;
        if let Some(hooks) = self.generic_hooks.get(&func) {
            for (_, hook_timing, callback) in hooks {
                if *hook_timing == timing {
                    let res = callback(entity_idx, timing);
                    if res.is_superceded() {
                        return res;
                    }
                    if res == HookResult::Handled {
                        final_result = HookResult::Handled;
                    }
                }
            }
        }
        final_result
    }
}

static REGISTRY: std::sync::LazyLock<RwLock<EntityHookRegistry>> =
    std::sync::LazyLock::new(|| RwLock::new(EntityHookRegistry::default()));

/// Global accessor for the entity hook registry.
pub fn entity_hooks() -> &'static RwLock<EntityHookRegistry> {
    &REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_damage_hook_mutation_and_supercede() {
        let mut reg = EntityHookRegistry::new();

        // Register Pre-hook that modifies damage (e.g. VIP 1.5x damage boost)
        reg.register_take_damage(None, HookTiming::Pre, |ctx, _timing| {
            if ctx.attacker == 1 {
                ctx.damage *= 1.5;
            }
            HookResult::Handled
        });

        // Register Pre-hook that blocks damage for victim 2 (GodMode)
        reg.register_take_damage(None, HookTiming::Pre, |ctx, _timing| {
            if ctx.victim == 2 {
                return HookResult::Supercede(0);
            }
            HookResult::Ignored
        });

        let mut normal_ctx = TakeDamageContext::new(3, 1, 1, 100.0, 0);
        let res = reg.dispatch_take_damage(&mut normal_ctx, HookTiming::Pre);
        assert_eq!(res, HookResult::Handled);
        assert_eq!(normal_ctx.damage, 150.0);

        let mut godmode_ctx = TakeDamageContext::new(2, 1, 1, 100.0, 0);
        let res = reg.dispatch_take_damage(&mut godmode_ctx, HookTiming::Pre);
        assert_eq!(res, HookResult::Supercede(0));
    }

    #[test]
    fn test_take_damage_commutative_modifiers_and_blackboard() {
        let mut reg = EntityHookRegistry::new();

        // Hook 1: Adds VIP perk multiplier 1.5 and writes to blackboard
        reg.register_take_damage(None, HookTiming::Pre, |ctx, _timing| {
            ctx.modifiers.add_multiplier("vip_perk", 1.5);
            ctx.blackboard.set_bool("is_vip_shot", true);
            HookResult::Handled
        });

        // Hook 2: Adds Clan perk multiplier 1.2 and reads from blackboard
        reg.register_take_damage(None, HookTiming::Pre, |ctx, _timing| {
            if ctx.blackboard.get_bool("is_vip_shot") == Some(true) {
                ctx.modifiers.add_flat("combo_bonus", 10.0);
            }
            ctx.modifiers.add_multiplier("clan_perk", 1.2);
            HookResult::Handled
        });

        let mut ctx = TakeDamageContext::new(5, 1, 1, 100.0, 0);
        let res = reg.dispatch_take_damage(&mut ctx, HookTiming::Pre);
        assert_eq!(res, HookResult::Handled);

        // (100 + 10) * 1.5 * 1.2 = 110 * 1.8 = 198.0
        assert!((ctx.damage - 198.0).abs() < 1e-4);
        assert!(ctx.modifiers.has_tag("vip_perk"));
        assert!(ctx.modifiers.has_tag("clan_perk"));
        assert!(ctx.modifiers.has_tag("combo_bonus"));
        assert_eq!(ctx.blackboard.get_bool("is_vip_shot"), Some(true));
    }

    #[test]
    fn test_generic_entity_hook_dispatch() {
        let mut reg = EntityHookRegistry::new();
        let was_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let was_called_clone = was_called.clone();

        reg.register_hook(
            VTableFunc::ResetMaxSpeed,
            None,
            HookTiming::Post,
            move |idx, _timing| {
                if idx == 1 {
                    was_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                }
                HookResult::Handled
            },
        );

        let res = reg.dispatch_generic(VTableFunc::ResetMaxSpeed, 1, HookTiming::Post);
        assert_eq!(res, HookResult::Handled);
        assert!(was_called.load(std::sync::atomic::Ordering::SeqCst));
    }
}
