//! Dual-tier combat hook bridge (`CombatBridge`) for GoldSrc using `stitch-rs`.
//!
//! Provides two-tier interception:
//! - Tier 1: ReGameDLL API hooks (direct API hookchains when ReGameDLL is active)
//! - Tier 2: Dynamic C++ VTable virtual function hooking fallback
//!
//! Evaluates hooks through a monomorphic U-Cycle pipeline implementing The Sewing Machine Architecture:
//! - Descent (`on_enter`): Filter & Protection checks with immediate short-circuit/halt (`FlowControl::Halt`).
//! - Puncture point (`TerminalHandler`): Mutation and algebraic damage calculation.
//! - Ascent (`on_exit`): Observations, metrics, sound, visual feedback, and post-strike telemetry.

use crate::hooks::entity::{KilledContext, TakeDamageContext, entity_hooks};
use crate::hooks::types::{HookResult, HookTiming};
use goldsrc_api::dag::EventPhase;
use std::sync::{Arc, LazyLock, RwLock};
use stitch_rs::flow::FlowControl;

/// Operational tier active for combat interception.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CombatTier {
    /// Tier 1: Native ReGameDLL extended hookchain API.
    Tier1ReGame,
    /// Tier 2: C++ VTable virtual function table hooking fallback.
    Tier2VTable,
}

/// Abstract take damage interceptor middleware based on `stitch-rs`.
pub trait TakeDamageLayer: Send + Sync {
    fn on_enter(&self, ctx: &mut TakeDamageContext) -> FlowControl<(), (), HookResult<i32>>;
    fn on_exit(&self, ctx: &mut TakeDamageContext, outcome: &mut Result<(), HookResult<i32>>);
}

/// Abstract killed interceptor middleware based on `stitch-rs`.
pub trait KilledLayer: Send + Sync {
    fn on_enter(&self, ctx: &KilledContext) -> FlowControl<(), (), HookResult<()>>;
    fn on_exit(&self, ctx: &KilledContext, outcome: &mut Result<(), HookResult<()>>);
}

/// Functional adapter implementing `TakeDamageLayer`.
struct FnTakeDamageLayer<F1, F2> {
    filter_or_handle: F1,
    observe: Option<F2>,
}

impl<F1, F2> TakeDamageLayer for FnTakeDamageLayer<F1, F2>
where
    F1: Fn(&mut TakeDamageContext) -> HookResult<i32> + Send + Sync,
    F2: Fn(&TakeDamageContext) + Send + Sync,
{
    fn on_enter(&self, ctx: &mut TakeDamageContext) -> FlowControl<(), (), HookResult<i32>> {
        let res = (self.filter_or_handle)(ctx);
        if res.is_superceded() {
            FlowControl::Halt(res)
        } else {
            FlowControl::Proceed(())
        }
    }

    fn on_exit(&self, ctx: &mut TakeDamageContext, outcome: &mut Result<(), HookResult<i32>>) {
        if outcome.is_ok()
            && let Some(ref obs) = self.observe
        {
            obs(ctx);
        }
    }
}

/// Dynamic SMA Pipeline Chain for Combat Events.
#[derive(Default)]
struct CombatPipelineRegistry {
    damage_layers: Vec<Arc<dyn TakeDamageLayer>>,
    killed_layers: Vec<Arc<dyn KilledLayer>>,
}

static COMBAT_REGISTRY: LazyLock<RwLock<CombatPipelineRegistry>> =
    LazyLock::new(|| RwLock::new(CombatPipelineRegistry::default()));

/// Central facade for combat event dispatching and registration.
pub struct CombatBridge;

impl CombatBridge {
    /// Resolves the currently active combat interception tier.
    pub fn active_tier() -> CombatTier {
        if crate::extension::extension_registry().is_available("regamedll", None) {
            CombatTier::Tier1ReGame
        } else {
            CombatTier::Tier2VTable
        }
    }

    /// Registers a custom `TakeDamageLayer` middleware.
    pub fn register_take_damage_layer(layer: Arc<dyn TakeDamageLayer>) {
        if let Ok(mut reg) = COMBAT_REGISTRY.write() {
            reg.damage_layers.push(layer);
        }
    }

    /// Registers a TakeDamage hook in a specific semantic event phase (backward-compatibility helper).
    pub fn register_take_damage<F>(phase: EventPhase, callback: F)
    where
        F: Fn(&mut TakeDamageContext, EventPhase) -> HookResult<i32> + Send + Sync + 'static,
    {
        match phase {
            EventPhase::Filter | EventPhase::Handle => {
                let layer = Arc::new(FnTakeDamageLayer {
                    filter_or_handle: move |ctx: &mut TakeDamageContext| callback(ctx, phase),
                    observe: None::<fn(&TakeDamageContext)>,
                });
                Self::register_take_damage_layer(layer);
            }
            EventPhase::Observe => {
                let layer = Arc::new(FnTakeDamageLayer {
                    filter_or_handle: |_ctx: &mut TakeDamageContext| HookResult::Ignored,
                    observe: Some(move |ctx: &TakeDamageContext| {
                        let mut copy = ctx.clone();
                        let _ = callback(&mut copy, EventPhase::Observe);
                    }),
                });
                Self::register_take_damage_layer(layer);
            }
        }
    }

    /// Registers a Killed hook in a specific semantic event phase.
    pub fn register_killed<F>(phase: EventPhase, callback: F)
    where
        F: Fn(&KilledContext, EventPhase) -> HookResult<()> + Send + Sync + 'static,
    {
        struct FnKilledLayer<F> {
            cb: F,
            phase: EventPhase,
        }
        impl<F> KilledLayer for FnKilledLayer<F>
        where
            F: Fn(&KilledContext, EventPhase) -> HookResult<()> + Send + Sync,
        {
            fn on_enter(&self, ctx: &KilledContext) -> FlowControl<(), (), HookResult<()>> {
                if self.phase != EventPhase::Observe {
                    let res = (self.cb)(ctx, self.phase);
                    if res.is_superceded() {
                        return FlowControl::Halt(res);
                    }
                }
                FlowControl::Proceed(())
            }

            fn on_exit(&self, ctx: &KilledContext, outcome: &mut Result<(), HookResult<()>>) {
                if outcome.is_ok() && self.phase == EventPhase::Observe {
                    let _ = (self.cb)(ctx, self.phase);
                }
            }
        }

        if let Ok(mut reg) = COMBAT_REGISTRY.write() {
            reg.killed_layers.push(Arc::new(FnKilledLayer {
                cb: callback,
                phase,
            }));
        }
    }

    /// Dispatches a `TakeDamage` event through the complete SMA pipeline.
    /// Returns the final hook verdict and the calculated mutated damage.
    pub fn dispatch_take_damage(
        victim: i32,
        inflictor: i32,
        attacker: i32,
        damage: f32,
        bits_damage_type: i32,
    ) -> (HookResult<i32>, f32) {
        let mut ctx = TakeDamageContext::new(victim, inflictor, attacker, damage, bits_damage_type);

        // 1. Fetch registered layers snapshot
        let layers = if let Ok(reg) = COMBAT_REGISTRY.read() {
            reg.damage_layers.clone()
        } else {
            Vec::new()
        };

        // 2. Execute SMA U-Cycle traversal: Descent (on_enter) -> Puncture -> Ascent (on_exit)
        let mut halted_res = None;
        let mut entered_count = 0;

        for layer in &layers {
            match layer.on_enter(&mut ctx) {
                FlowControl::Proceed(()) => {
                    ctx.sync_damage();
                    entered_count += 1;
                }
                FlowControl::ShortCircuit(()) => {
                    ctx.sync_damage();
                    entered_count += 1;
                    break;
                }
                FlowControl::Halt(res) => {
                    halted_res = Some(res);
                    break;
                }
            }
        }

        // 3. Ascent phase: unwind executed layers in reverse order for observation
        let mut outcome = match halted_res {
            Some(res) => Err(res),
            None => Ok(()),
        };

        for layer in layers.iter().take(entered_count).rev() {
            layer.on_exit(&mut ctx, &mut outcome);
        }

        if let Err(res) = outcome {
            return (res, 0.0);
        }

        // 4. Legacy / Direct VTable EntityHookRegistry Dispatch
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

    /// Dispatches a `Killed` event through the complete SMA pipeline.
    pub fn dispatch_killed(victim: i32, attacker: i32, gib_mode: i32) -> HookResult<()> {
        let ctx = KilledContext {
            victim,
            attacker,
            gib_mode,
        };

        let layers = if let Ok(reg) = COMBAT_REGISTRY.read() {
            reg.killed_layers.clone()
        } else {
            Vec::new()
        };

        let mut halted_res = None;
        let mut entered_count = 0;

        for layer in &layers {
            match layer.on_enter(&ctx) {
                FlowControl::Proceed(()) => {
                    entered_count += 1;
                }
                FlowControl::ShortCircuit(()) => {
                    entered_count += 1;
                    break;
                }
                FlowControl::Halt(res) => {
                    halted_res = Some(res);
                    break;
                }
            }
        }

        let mut outcome = match halted_res {
            Some(res) => Err(res),
            None => Ok(()),
        };

        for layer in layers.iter().take(entered_count).rev() {
            layer.on_exit(&ctx, &mut outcome);
        }

        if let Err(res) = outcome {
            return res;
        }

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
