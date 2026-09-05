//! Generic Reactive Rule & Provider Engine.
//!
//! Provides decoupled condition evaluators ([`RuleCondition`]) and action
//! executors ([`RuleAction`]) registered in a pluggable [`RuleRegistry`].

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// Scope or domain that triggers rule evaluation.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum RuleScope {
    /// Evaluates all rules regardless of scope (e.g. initial boot or configuration hot-reload).
    All,
    /// Triggered on server initialization, map load, or changelevel.
    MapChange,
    /// Triggered when the number of players changes (client connect, disconnect, putinserver).
    PlayerCount,
    /// Triggered on recurring or scheduled time intervals.
    TimeSchedule,
    /// Triggered on server CVar change.
    CvarChange(String),
    /// Extensible custom scope for plugin-defined events (e.g. "player_capability").
    Custom(String),
}

impl RuleScope {
    /// Parses a string representation into a [`RuleScope`].
    pub fn parse(s: &str) -> Self {
        let trimmed = s.trim();
        match trimmed.to_ascii_lowercase().as_str() {
            "all" | "*" => Self::All,
            "map" | "map_change" | "mapchange" => Self::MapChange,
            "player" | "players" | "player_count" | "playercount" => Self::PlayerCount,
            "time" | "schedule" | "time_schedule" | "timeschedule" => Self::TimeSchedule,
            _ => {
                if let Some(cvar) = trimmed.strip_prefix("cvar:") {
                    Self::CvarChange(cvar.trim().to_string())
                } else {
                    Self::Custom(trimmed.to_string())
                }
            }
        }
    }

    /// Returns the canonical string identifier for the scope.
    pub fn as_str(&self) -> &str {
        match self {
            Self::All => "all",
            Self::MapChange => "map_change",
            Self::PlayerCount => "player_count",
            Self::TimeSchedule => "time_schedule",
            Self::CvarChange(c) => c.as_str(),
            Self::Custom(c) => c.as_str(),
        }
    }
}

impl std::fmt::Display for RuleScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A pluggable condition evaluator for a specific context `Ctx`.
pub trait RuleCondition<Ctx>: Send + Sync {
    /// Unique name of this condition provider (e.g. "map", "players", "cvar", "time").
    fn name(&self) -> &str;

    /// Evaluates whether the condition is met given the current `ctx` and TOML specification.
    fn evaluate(&self, ctx: &Ctx, value: &toml::Value) -> bool;

    /// Returns the default trigger scopes for this condition.
    /// If empty, the condition is evaluated across all scopes.
    fn scopes(&self) -> Vec<RuleScope> {
        Vec::new()
    }
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

/// A parsed reactive rule consisting of conditions, actions, and trigger scopes.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Human-readable name/description of the rule.
    pub name: String,
    /// Map of condition name -> TOML value (sorted deterministically).
    pub when: BTreeMap<String, toml::Value>,
    /// Map of action name -> TOML value (sorted deterministically).
    pub action: BTreeMap<String, toml::Value>,
    /// Optional explicit trigger scopes for this rule. If empty, derived from conditions.
    pub scopes: Vec<RuleScope>,
}

impl Rule {
    /// Creates a new rule with automatic trigger scope derivation.
    pub fn new<S: Into<String>>(
        name: S,
        when: BTreeMap<String, toml::Value>,
        action: BTreeMap<String, toml::Value>,
    ) -> Self {
        Self {
            name: name.into(),
            when,
            action,
            scopes: Vec::new(),
        }
    }

    /// Creates a new rule with explicit trigger scopes.
    pub fn with_scopes<S: Into<String>>(
        name: S,
        when: BTreeMap<String, toml::Value>,
        action: BTreeMap<String, toml::Value>,
        scopes: Vec<RuleScope>,
    ) -> Self {
        Self {
            name: name.into(),
            when,
            action,
            scopes,
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

    /// Evaluates a single condition or boolean operator recursively.
    pub fn evaluate_condition(&self, ctx: &Ctx, cond_name: &str, cond_val: &toml::Value) -> bool {
        match cond_name {
            "all_of" => {
                if let toml::Value::Array(items) = cond_val {
                    items.iter().all(|item| {
                        if let toml::Value::Table(tbl) = item {
                            tbl.iter().all(|(k, v)| self.evaluate_condition(ctx, k, v))
                        } else {
                            false
                        }
                    })
                } else if let toml::Value::Table(tbl) = cond_val {
                    tbl.iter().all(|(k, v)| self.evaluate_condition(ctx, k, v))
                } else {
                    false
                }
            }
            "any_of" => {
                if let toml::Value::Array(items) = cond_val {
                    items.iter().any(|item| {
                        if let toml::Value::Table(tbl) = item {
                            tbl.iter().all(|(k, v)| self.evaluate_condition(ctx, k, v))
                        } else {
                            false
                        }
                    })
                } else if let toml::Value::Table(tbl) = cond_val {
                    tbl.iter().any(|(k, v)| self.evaluate_condition(ctx, k, v))
                } else {
                    false
                }
            }
            "none_of" => {
                if let toml::Value::Array(items) = cond_val {
                    !items.iter().any(|item| {
                        if let toml::Value::Table(tbl) = item {
                            tbl.iter().all(|(k, v)| self.evaluate_condition(ctx, k, v))
                        } else {
                            false
                        }
                    })
                } else if let toml::Value::Table(tbl) = cond_val {
                    !tbl.iter().all(|(k, v)| self.evaluate_condition(ctx, k, v))
                } else {
                    false
                }
            }
            _ => {
                if let Some(cond) = self.registry.get_condition(cond_name) {
                    cond.evaluate(ctx, cond_val)
                } else {
                    // Unknown condition fails closed
                    false
                }
            }
        }
    }

    /// Resolves the active trigger scopes for a rule.
    /// If explicit scopes are defined on the rule, they are used.
    /// Otherwise, extracts default scopes from registered conditions in `when`.
    /// If no conditions declare a scope, defaults to `[RuleScope::MapChange]`.
    pub fn resolve_rule_scopes(&self, rule: &Rule) -> Vec<RuleScope> {
        if !rule.scopes.is_empty() {
            return rule.scopes.clone();
        }

        let mut scopes = BTreeSet::new();
        for cond_name in rule.when.keys() {
            if let Some(cond) = self.registry.get_condition(cond_name) {
                for s in cond.scopes() {
                    scopes.insert(s);
                }
            }
        }

        if scopes.is_empty() {
            vec![RuleScope::MapChange]
        } else {
            scopes.into_iter().collect()
        }
    }

    /// Evaluates rules matching a specific [`RuleScope`] (or all rules if `scope == &RuleScope::All`)
    /// against `ctx` and executes actions for satisfied rules.
    /// Returns a vector of results `(rule_name, Result<(), Vec<String>>)`.
    pub fn evaluate_and_execute_scope(
        &self,
        ctx: &mut Ctx,
        scope: &RuleScope,
    ) -> Vec<(String, Result<(), Vec<String>>)> {
        let mut results = Vec::new();

        for rule in &self.rules {
            if *scope != RuleScope::All {
                let rule_scopes = self.resolve_rule_scopes(rule);
                if !rule_scopes.contains(scope) {
                    continue;
                }
            }

            // All conditions in `when` must evaluate to true (AND logic)
            let mut satisfied = true;
            for (cond_name, cond_val) in &rule.when {
                if !self.evaluate_condition(ctx, cond_name, cond_val) {
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

    /// Evaluates all rules against `ctx` and executes actions for satisfied rules.
    /// Equivalent to calling `evaluate_and_execute_scope(ctx, &RuleScope::All)`.
    pub fn evaluate_and_execute(&self, ctx: &mut Ctx) -> Vec<(String, Result<(), Vec<String>>)> {
        self.evaluate_and_execute_scope(ctx, &RuleScope::All)
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
        fn scopes(&self) -> Vec<RuleScope> {
            vec![RuleScope::MapChange]
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
        fn scopes(&self) -> Vec<RuleScope> {
            vec![RuleScope::PlayerCount]
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

    #[test]
    fn test_rule_engine_scoping() {
        let mut registry = RuleRegistry::new();
        registry.register_condition(MapCondition);
        registry.register_condition(PlayersCondition);
        registry.register_action(SetGravityAction);
        registry.register_action(BroadcastAction);

        let mut when_map = BTreeMap::new();
        when_map.insert(
            "map".to_string(),
            toml::Value::String("de_dust2".to_string()),
        );
        let mut act_map = BTreeMap::new();
        act_map.insert("set_gravity".to_string(), toml::Value::Integer(500));
        let map_rule = Rule::new("auto_pause_dust2", when_map, act_map);

        let mut when_players = BTreeMap::new();
        when_players.insert("players".to_string(), toml::Value::Integer(10));
        let mut act_players = BTreeMap::new();
        act_players.insert(
            "broadcast".to_string(),
            toml::Value::String("Full lobby!".to_string()),
        );
        let players_rule = Rule::new("full_lobby", when_players, act_players);

        let engine = RuleEngine::new(registry, vec![map_rule, players_rule]);

        let mut ctx = TestContext {
            map: "de_dust2".to_string(),
            players: 12,
            gravity: 800,
            messages: Vec::new(),
        };

        // When evaluating PlayerCount scope (e.g. client_connect/disconnect), map rule must NOT run
        let results = engine.evaluate_and_execute_scope(&mut ctx, &RuleScope::PlayerCount);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "full_lobby");
        assert_eq!(ctx.messages, vec!["Full lobby!"]);
        assert_eq!(ctx.gravity, 800); // map rule was NOT executed

        // When evaluating MapChange scope, only map rule runs
        ctx.messages.clear();
        let results_map = engine.evaluate_and_execute_scope(&mut ctx, &RuleScope::MapChange);
        assert_eq!(results_map.len(), 1);
        assert_eq!(results_map[0].0, "auto_pause_dust2");
        assert_eq!(ctx.gravity, 500);
        assert!(ctx.messages.is_empty());
    }
}
