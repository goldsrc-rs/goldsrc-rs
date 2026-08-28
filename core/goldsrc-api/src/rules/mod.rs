//! Generic Reactive Rule & Provider Engine.
//!
//! Provides decoupled condition evaluators ([`RuleCondition`]) and action
//! executors ([`RuleAction`]) registered in a pluggable [`RuleRegistry`].

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

/// A pluggable condition evaluator for a specific context `Ctx`.
pub trait RuleCondition<Ctx>: Send + Sync {
    /// Unique name of this condition provider (e.g. "map", "players", "cvar", "time").
    fn name(&self) -> &str;

    /// Evaluates whether the condition is met given the current `ctx` and TOML specification.
    fn evaluate(&self, ctx: &Ctx, value: &toml::Value) -> bool;
}

/// A pluggable action executor for a specific context `Ctx`.
pub trait RuleAction<Ctx>: Send + Sync {
    /// Unique name of this action provider (e.g. "pause", "unpause", "set_cvar", "exec").
    fn name(&self) -> &str;

    /// Executes the action against `ctx` given the TOML specification.
    fn execute(&self, ctx: &mut Ctx, value: &toml::Value) -> Result<(), String>;
}

/// Registry of condition evaluators and action executors for context `Ctx`.
pub struct RuleRegistry<Ctx> {
    conditions: HashMap<String, Arc<dyn RuleCondition<Ctx>>>,
    actions: HashMap<String, Arc<dyn RuleAction<Ctx>>>,
}

impl<Ctx> Default for RuleRegistry<Ctx> {
    fn default() -> Self {
        Self {
            conditions: HashMap::new(),
            actions: HashMap::new(),
        }
    }
}

impl<Ctx> RuleRegistry<Ctx> {
    /// Creates a new empty rule registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a condition evaluator.
    pub fn register_condition<C: RuleCondition<Ctx> + 'static>(&mut self, condition: C) {
        self.conditions
            .insert(condition.name().to_string(), Arc::new(condition));
    }

    /// Registers an action executor.
    pub fn register_action<A: RuleAction<Ctx> + 'static>(&mut self, action: A) {
        self.actions
            .insert(action.name().to_string(), Arc::new(action));
    }

    /// Looks up a condition evaluator by name.
    pub fn get_condition(&self, name: &str) -> Option<&Arc<dyn RuleCondition<Ctx>>> {
        self.conditions.get(name)
    }

    /// Looks up an action executor by name.
    pub fn get_action(&self, name: &str) -> Option<&Arc<dyn RuleAction<Ctx>>> {
        self.actions.get(name)
    }
}

/// A parsed reactive rule consisting of conditions and actions.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Human-readable name/description of the rule.
    pub name: String,
    /// Map of condition name -> TOML value (sorted deterministically).
    pub when: BTreeMap<String, toml::Value>,
    /// Map of action name -> TOML value (sorted deterministically).
    pub action: BTreeMap<String, toml::Value>,
}

impl Rule {
    /// Creates a new rule.
    pub fn new<S: Into<String>>(
        name: S,
        when: BTreeMap<String, toml::Value>,
        action: BTreeMap<String, toml::Value>,
    ) -> Self {
        Self {
            name: name.into(),
            when,
            action,
        }
    }
}

/// Reactive rule engine that evaluates rules against a context and executes actions.
pub struct RuleEngine<Ctx> {
    registry: RuleRegistry<Ctx>,
    rules: Vec<Rule>,
}

impl<Ctx> RuleEngine<Ctx> {
    /// Creates a new rule engine with the specified registry and rule set.
    pub fn new(registry: RuleRegistry<Ctx>, rules: Vec<Rule>) -> Self {
        Self { registry, rules }
    }

    /// Returns a reference to the rule registry.
    pub fn registry(&self) -> &RuleRegistry<Ctx> {
        &self.registry
    }

    /// Returns a mutable reference to the rule registry.
    pub fn registry_mut(&mut self) -> &mut RuleRegistry<Ctx> {
        &mut self.registry
    }

    /// Returns a reference to the active rules.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Sets the active rules.
    pub fn set_rules(&mut self, rules: Vec<Rule>) {
        self.rules = rules;
    }

    /// Evaluates all rules against `ctx` and executes actions for satisfied rules.
    /// Returns a vector of results `(rule_name, Result<(), Vec<String>>)`.
    pub fn evaluate_and_execute(&self, ctx: &mut Ctx) -> Vec<(String, Result<(), Vec<String>>)> {
        let mut results = Vec::new();

        for rule in &self.rules {
            // All conditions in `when` must evaluate to true (AND logic)
            let mut satisfied = true;
            for (cond_name, cond_val) in &rule.when {
                if let Some(cond) = self.registry.get_condition(cond_name) {
                    if !cond.evaluate(ctx, cond_val) {
                        satisfied = false;
                        break;
                    }
                } else {
                    // Unknown condition fails closed
                    satisfied = false;
                    break;
                }
            }

            if satisfied {
                let mut errors = Vec::new();
                for (act_name, act_val) in &rule.action {
                    if let Some(action) = self.registry.get_action(act_name) {
                        if let Err(e) = action.execute(ctx, act_val) {
                            errors.push(format!("Action '{}' failed: {}", act_name, e));
                        }
                    } else {
                        errors.push(format!("Unknown action executor '{}'", act_name));
                    }
                }

                if errors.is_empty() {
                    results.push((rule.name.clone(), Ok(())));
                } else {
                    results.push((rule.name.clone(), Err(errors)));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestContext {
        map: String,
        players: usize,
        gravity: i32,
        messages: Vec<String>,
    }

    struct MapCondition;
    impl RuleCondition<TestContext> for MapCondition {
        fn name(&self) -> &str {
            "map"
        }
        fn evaluate(&self, ctx: &TestContext, value: &toml::Value) -> bool {
            match value {
                toml::Value::String(s) => ctx.map == *s,
                toml::Value::Array(arr) => arr.iter().any(|v| match v {
                    toml::Value::String(s) => ctx.map == *s,
                    _ => false,
                }),
                _ => false,
            }
        }
    }

    struct PlayersCondition;
    impl RuleCondition<TestContext> for PlayersCondition {
        fn name(&self) -> &str {
            "players"
        }
        fn evaluate(&self, ctx: &TestContext, value: &toml::Value) -> bool {
            if let toml::Value::Integer(n) = value {
                ctx.players >= *n as usize
            } else {
                false
            }
        }
    }

    struct SetGravityAction;
    impl RuleAction<TestContext> for SetGravityAction {
        fn name(&self) -> &str {
            "set_gravity"
        }
        fn execute(&self, ctx: &mut TestContext, value: &toml::Value) -> Result<(), String> {
            if let toml::Value::Integer(g) = value {
                ctx.gravity = *g as i32;
                Ok(())
            } else {
                Err("Expected integer gravity".to_string())
            }
        }
    }

    struct BroadcastAction;
    impl RuleAction<TestContext> for BroadcastAction {
        fn name(&self) -> &str {
            "broadcast"
        }
        fn execute(&self, ctx: &mut TestContext, value: &toml::Value) -> Result<(), String> {
            if let toml::Value::String(msg) = value {
                ctx.messages.push(msg.clone());
                Ok(())
            } else {
                Err("Expected string message".to_string())
            }
        }
    }

    #[test]
    fn test_rule_engine_evaluation() {
        let mut registry = RuleRegistry::new();
        registry.register_condition(MapCondition);
        registry.register_condition(PlayersCondition);
        registry.register_action(SetGravityAction);
        registry.register_action(BroadcastAction);

        let mut when1 = BTreeMap::new();
        when1.insert(
            "map".to_string(),
            toml::Value::String("de_dust2".to_string()),
        );
        when1.insert("players".to_string(), toml::Value::Integer(10));

        let mut action1 = BTreeMap::new();
        action1.insert("set_gravity".to_string(), toml::Value::Integer(600));
        action1.insert(
            "broadcast".to_string(),
            toml::Value::String("Dust2 Mode!".to_string()),
        );

        let rule1 = Rule::new("dust2_rule", when1, action1);

        let engine = RuleEngine::new(registry, vec![rule1]);

        let mut ctx = TestContext {
            map: "de_dust2".to_string(),
            players: 12,
            gravity: 800,
            messages: Vec::new(),
        };

        let results = engine.evaluate_and_execute(&mut ctx);
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_ok());
        assert_eq!(ctx.gravity, 600);
        assert_eq!(ctx.messages, vec!["Dust2 Mode!"]);
    }
}
