//! Dual-tier combat hook bridge (`CombatBridge`) for GoldSrc.
//!
//! Provides two-tier interception:
//! - Tier 1: ReGameDLL API hooks (direct API hookchains when ReGameDLL is active)
//! - Tier 2: Dynamic C++ VTable virtual function hooking fallback
//!
//! Evaluates hooks through a phased pipeline:
//! `EventPhase::Filter` -> `EventPhase::Handle` -> `EventPhase::Observe`
//! with order-independent commutative algebraic modifiers.

use crate::hooks::entity::{KilledContext, TakeDamageContext, entity_hooks};
use crate::hooks::types::{HookResult, HookTiming};
use crate::reapi::ReApiBridge;
use goldsrc_api::dag::EventPhase;
use std::sync::{LazyLock, RwLock};

/// Operational tier active for combat interception.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CombatTier {
    /// Tier 1: Native ReGameDLL extended hookchain API.
    Tier1ReGame,
    /// Tier 2: C++ VTable virtual function table hooking fallback.
    Tier2VTable,
}

/// Callback signature for phased TakeDamage interception.
pub type PhasedTakeDamageHook =
    Box<dyn Fn(&mut TakeDamageContext, EventPhase) -> HookResult<i32> + Send + Sync + 'static>;

/// Callback signature for phased Killed interception.
pub type PhasedKilledHook =
    Box<dyn Fn(&KilledContext, EventPhase) -> HookResult<()> + Send + Sync + 'static>;

/// Internal registry for phased combat handlers.
#[derive(Default)]
struct PhasedCombatRegistry {
    filter_damage: Vec<PhasedTakeDamageHook>,
    handle_damage: Vec<PhasedTakeDamageHook>,
    observe_damage: Vec<PhasedTakeDamageHook>,

    filter_killed: Vec<PhasedKilledHook>,
    handle_killed: Vec<PhasedKilledHook>,
    observe_killed: Vec<PhasedKilledHook>,
}

static COMBAT_REGISTRY: LazyLock<RwLock<PhasedCombatRegistry>> =
    LazyLock::new(|| RwLock::new(PhasedCombatRegistry::default()));

/// Central facade for combat event dispatching and registration.
pub struct CombatBridge;

impl CombatBridge {
    /// Resolves the currently active combat interception tier.
    pub fn active_tier() -> CombatTier {
        if ReApiBridge::status().regamedll_active {
            CombatTier::Tier1ReGame
        } else {
            CombatTier::Tier2VTable
        }
    }

    /// Registers a TakeDamage hook in a specific semantic event phase.
    pub fn register_take_damage<F>(phase: EventPhase, callback: F)
    where
        F: Fn(&mut TakeDamageContext, EventPhase) -> HookResult<i32> + Send + Sync + 'static,
    {
        if let Ok(mut reg) = COMBAT_REGISTRY.write() {
            let boxed = Box::new(callback);
            match phase {
                EventPhase::Filter => reg.filter_damage.push(boxed),
                EventPhase::Handle => reg.handle_damage.push(boxed),
                EventPhase::Observe => reg.observe_damage.push(boxed),
            }
        }
    }

    /// Registers a Killed hook in a specific semantic event phase.
    pub fn register_killed<F>(phase: EventPhase, callback: F)
    where
        F: Fn(&KilledContext, EventPhase) -> HookResult<()> + Send + Sync + 'static,
    {
        if let Ok(mut reg) = COMBAT_REGISTRY.write() {
            let boxed = Box::new(callback);
            match phase {
                EventPhase::Filter => reg.filter_killed.push(boxed),
                EventPhase::Handle => reg.handle_killed.push(boxed),
                EventPhase::Observe => reg.observe_killed.push(boxed),
            }
        }
    }

    /// Dispatches a `TakeDamage` event through the complete phased pipeline.
    /// Returns the final hook verdict and the calculated mutated damage.
    pub fn dispatch_take_damage(
        victim: i32,
        inflictor: i32,
        attacker: i32,
        damage: f32,
        bits_damage_type: i32,
    ) -> (HookResult<i32>, f32) {
        let mut ctx = TakeDamageContext::new(victim, inflictor, attacker, damage, bits_damage_type);

        // 1. Phased Pipeline Execution
        if let Ok(reg) = COMBAT_REGISTRY.read() {
            // Phase 1: Filter (protection, early cancellation, godmode)
            for hook in &reg.filter_damage {
                let res = hook(&mut ctx, EventPhase::Filter);
                if res.is_superceded() {
                    return (res, 0.0);
                }
            }

            // Phase 2: Handle (commutative damage bonuses, reductions, multipliers)
            for hook in &reg.handle_damage {
                let res = hook(&mut ctx, EventPhase::Handle);
                ctx.sync_damage();
                if res.is_superceded() {
                    return (res, 0.0);
                }
            }

            // Phase 3: Observe (metrics, analytics, sound/visual feedback)
            for hook in &reg.observe_damage {
                let _ = hook(&mut ctx, EventPhase::Observe);
            }
        }

        // 2. Legacy / Direct VTable EntityHookRegistry Dispatch
        let vtable_res = if let Ok(reg) = entity_hooks().read() {
            reg.dispatch_take_damage(&mut ctx, HookTiming::Pre)
        } else {
            HookResult::Ignored
        };

        if vtable_res.is_superceded() {
            return (vtable_res, 0.0);
        }

        let final_damage = ctx.sync_damage();
        (vtable_res, final_damage)
    }

    /// Dispatches a `Killed` event through the complete phased pipeline.
    pub fn dispatch_killed(victim: i32, attacker: i32, gib_mode: i32) -> HookResult<()> {
        let ctx = KilledContext {
            victim,
            attacker,
            gib_mode,
        };

        // 1. Phased Pipeline Execution
        if let Ok(reg) = COMBAT_REGISTRY.read() {
            // Phase 1: Filter
            for hook in &reg.filter_killed {
                let res = hook(&ctx, EventPhase::Filter);
                if res.is_superceded() {
                    return res;
                }
            }

            // Phase 2: Handle
            for hook in &reg.handle_killed {
                let res = hook(&ctx, EventPhase::Handle);
                if res.is_superceded() {
                    return res;
                }
            }

            // Phase 3: Observe
            for hook in &reg.observe_killed {
                let _ = hook(&ctx, EventPhase::Observe);
            }
        }

        // 2. Legacy / Direct VTable EntityHookRegistry Dispatch
        if let Ok(reg) = entity_hooks().read() {
            reg.dispatch_killed(&ctx, HookTiming::Pre)
        } else {
            HookResult::Ignored
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_bridge_phased_damage_flow() {
        CombatBridge::register_take_damage(EventPhase::Filter, |ctx, _| {
            if ctx.victim == 99 {
                return HookResult::Supercede(0);
            }
            HookResult::Ignored
        });

        CombatBridge::register_take_damage(EventPhase::Handle, |ctx, _| {
            ctx.modifiers.add_multiplier("perk_2x", 2.0);
            HookResult::Handled
        });

        // Test normal victim with 2x multiplier
        let (res, dmg) = CombatBridge::dispatch_take_damage(1, 2, 2, 50.0, 0);
        assert!(!res.is_superceded());
        assert!((dmg - 100.0).abs() < 1e-4);

        // Test protected victim 99
        let (res, dmg) = CombatBridge::dispatch_take_damage(99, 2, 2, 50.0, 0);
        assert!(res.is_superceded());
        assert_eq!(dmg, 0.0);
    }
}
