//! Commutative State Modifiers and Typed Context Blackboard.
//!
//! Provides order-independent state mutations (e.g. damage calculation)
//! and a type-safe property bag for inter-plugin event/hook contexts.
//!
//! # Commutativity Invariant
//!
//! Unlike legacy systems where plugin execution order can overwrite or wipe out
//! values (e.g. `damage = damage * 1.5` clobbering another plugin's reduction),
//! `CommutativeModifier` uses an algebraic evaluation model:
//!
//! $$\text{Final} = \max\left(0.0, (\text{base} + \sum \text{flat\_bonuses}) \times \prod \text{multipliers} - \sum \text{reductions}\right)$$
//!
//! Because addition and multiplication are commutative:
//! $$A \times B = B \times A, \quad A + B = B + A$$
//! The order in which plugins attach perks, debuffs, or resistances does not produce
//! conflicting or non-deterministic outcomes.

use std::collections::HashMap;

/// An individual tagged contribution to a modifier calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifierContribution {
    /// Identifying tag or plugin label (e.g. `"vip_headshot"`, `"kevlar_armor"`).
    pub tag: String,
    /// Numeric contribution value.
    pub value: f32,
}

/// Commutative, order-independent state mutation calculator.
#[derive(Debug, Clone, PartialEq)]
pub struct CommutativeModifier {
    /// Base original value (e.g. weapon base damage).
    pub base: f32,
    /// Additive flat bonuses (e.g. `+10.0` fire damage).
    pub flat_bonuses: Vec<ModifierContribution>,
    /// Multiplicative factors (e.g. `1.5` critical hit, `1.2` rage potion).
    pub multipliers: Vec<ModifierContribution>,
    /// Subtractive reductions or armor absorptions (e.g. `-15.0` shield).
    pub reductions: Vec<ModifierContribution>,
    /// Whether the final value is completely negated/blocked (e.g. invulnerability/godmode).
    pub is_blocked: bool,
}

impl Default for CommutativeModifier {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl CommutativeModifier {
    /// Creates a new `CommutativeModifier` with the specified base value.
    pub fn new(base: f32) -> Self {
        Self {
            base,
            flat_bonuses: Vec::new(),
            multipliers: Vec::new(),
            reductions: Vec::new(),
            is_blocked: false,
        }
    }

    /// Adds a flat bonus to the base value before multipliers are applied.
    pub fn add_flat(&mut self, tag: impl Into<String>, amount: f32) -> &mut Self {
        self.flat_bonuses.push(ModifierContribution {
            tag: tag.into(),
            value: amount,
        });
        self
    }

    /// Adds a multiplicative factor. Multipliers compound commutatively via multiplication.
    pub fn add_multiplier(&mut self, tag: impl Into<String>, factor: f32) -> &mut Self {
        self.multipliers.push(ModifierContribution {
            tag: tag.into(),
            value: factor,
        });
        self
    }

    /// Adds a flat reduction subtracted after multipliers have been evaluated.
    pub fn add_reduction(&mut self, tag: impl Into<String>, amount: f32) -> &mut Self {
        self.reductions.push(ModifierContribution {
            tag: tag.into(),
            value: amount,
        });
        self
    }

    /// Fully blocks and nullifies the computed result (returns `0.0`).
    pub fn block(&mut self) -> &mut Self {
        self.is_blocked = true;
        self
    }

    /// Clears any blocking flag.
    pub fn unblock(&mut self) -> &mut Self {
        self.is_blocked = false;
        self
    }

    /// Returns `true` if the value has been fully blocked.
    pub fn is_blocked(&self) -> bool {
        self.is_blocked
    }

    /// Checks if a contribution with the specified tag exists in any category.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.flat_bonuses.iter().any(|c| c.tag == tag)
            || self.multipliers.iter().any(|c| c.tag == tag)
            || self.reductions.iter().any(|c| c.tag == tag)
    }

    /// Computes the final resulting value using the algebraic commutative model:
    ///
    /// $$\text{Result} = \max\left(0.0, (\text{base} + \sum \text{flat}) \times \prod \text{mult} - \sum \text{red}\right)$$
    pub fn compute(&self) -> f32 {
        if self.is_blocked {
            return 0.0;
        }

        let flat_sum: f32 = self.flat_bonuses.iter().map(|c| c.value).sum();
        let mult_prod: f32 = self.multipliers.iter().map(|c| c.value).product();
        let red_sum: f32 = self.reductions.iter().map(|c| c.value).sum();

        let total = (self.base + flat_sum) * mult_prod - red_sum;
        if total < 0.0 { 0.0 } else { total }
    }

    /// Returns a human-readable diagnostic breakdown of all applied contributions.
    pub fn explain(&self) -> String {
        if self.is_blocked {
            return format!("Base: {:.2} [BLOCKED] => 0.00", self.base);
        }

        let flat_str: Vec<String> = self
            .flat_bonuses
            .iter()
            .map(|c| format!("+{:.2} ({})", c.value, c.tag))
            .collect();
        let mult_str: Vec<String> = self
            .multipliers
            .iter()
            .map(|c| format!("x{:.2} ({})", c.value, c.tag))
            .collect();
        let red_str: Vec<String> = self
            .reductions
            .iter()
            .map(|c| format!("-{:.2} ({})", c.value, c.tag))
            .collect();

        format!(
            "Base: {:.2}, Flat: [{}], Mult: [{}], Reductions: [{}] => Final: {:.2}",
            self.base,
            flat_str.join(", "),
            mult_str.join(", "),
            red_str.join(", "),
            self.compute()
        )
    }
}

/// Primitive property value stored in a `TypedBlackboard`.
#[derive(Debug, Clone, PartialEq)]
pub enum BlackboardValue {
    /// Boolean flag.
    Bool(bool),
    /// 64-bit signed integer.
    Int(i64),
    /// 32-bit floating point number.
    Float(f32),
    /// UTF-8 string.
    String(String),
    /// Raw byte buffer.
    Bytes(Vec<u8>),
}

/// Dynamic, type-safe property bag for event/hook contexts.
///
/// Allows plugins to share auxiliary state and context without direct compile-time
/// dependencies or rigid struct coupling.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TypedBlackboard {
    properties: HashMap<String, BlackboardValue>,
}

impl TypedBlackboard {
    /// Creates a new empty blackboard.
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    /// Sets a boolean property.
    pub fn set_bool(&mut self, key: impl Into<String>, val: bool) {
        self.properties
            .insert(key.into(), BlackboardValue::Bool(val));
    }

    /// Gets a boolean property, returning `None` if missing or wrong type.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.properties.get(key) {
            Some(BlackboardValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Sets an integer property.
    pub fn set_int(&mut self, key: impl Into<String>, val: i64) {
        self.properties
            .insert(key.into(), BlackboardValue::Int(val));
    }

    /// Gets an integer property, returning `None` if missing or wrong type.
    pub fn get_int(&self, key: &str) -> Option<i64> {
        match self.properties.get(key) {
            Some(BlackboardValue::Int(i)) => Some(*i),
            _ => None,
        }
    }

    /// Sets a float property.
    pub fn set_float(&mut self, key: impl Into<String>, val: f32) {
        self.properties
            .insert(key.into(), BlackboardValue::Float(val));
    }

    /// Gets a float property, returning `None` if missing or wrong type.
    pub fn get_float(&self, key: &str) -> Option<f32> {
        match self.properties.get(key) {
            Some(BlackboardValue::Float(f)) => Some(*f),
            _ => None,
        }
    }

    /// Sets a string property.
    pub fn set_str(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.properties
            .insert(key.into(), BlackboardValue::String(val.into()));
    }

    /// Gets a string property slice, returning `None` if missing or wrong type.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.properties.get(key) {
            Some(BlackboardValue::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Sets a raw byte buffer property.
    pub fn set_bytes(&mut self, key: impl Into<String>, val: Vec<u8>) {
        self.properties
            .insert(key.into(), BlackboardValue::Bytes(val));
    }

    /// Gets a byte slice property, returning `None` if missing or wrong type.
    pub fn get_bytes(&self, key: &str) -> Option<&[u8]> {
        match self.properties.get(key) {
            Some(BlackboardValue::Bytes(b)) => Some(b.as_slice()),
            _ => None,
        }
    }

    /// Returns `true` if a property with the given key exists.
    pub fn contains_key(&self, key: &str) -> bool {
        self.properties.contains_key(key)
    }

    /// Removes a property from the blackboard.
    pub fn remove(&mut self, key: &str) -> Option<BlackboardValue> {
        self.properties.remove(key)
    }

    /// Clears all properties from the blackboard.
    pub fn clear(&mut self) {
        self.properties.clear();
    }

    /// Total number of properties in the blackboard.
    pub fn len(&self) -> usize {
        self.properties.len()
    }

    /// Returns `true` if the blackboard is empty.
    pub fn is_empty(&self) -> bool {
        self.properties.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commutative_multipliers_order_invariance() {
        // Order A: Clan perk (1.2x) then VIP bonus (1.5x)
        let mut mod_a = CommutativeModifier::new(50.0);
        mod_a.add_multiplier("clan_perk", 1.2);
        mod_a.add_multiplier("vip_bonus", 1.5);

        // Order B: VIP bonus (1.5x) then Clan perk (1.2x)
        let mut mod_b = CommutativeModifier::new(50.0);
        mod_b.add_multiplier("vip_bonus", 1.5);
        mod_b.add_multiplier("clan_perk", 1.2);

        assert!((mod_a.compute() - mod_b.compute()).abs() < 1e-5);
        assert_eq!(mod_a.compute(), 90.0);
        assert_eq!(mod_b.compute(), 90.0);
    }

    #[test]
    fn test_commutative_flat_and_reductions() {
        let mut modifier = CommutativeModifier::new(100.0);
        modifier
            .add_flat("fire_damage", 10.0)
            .add_flat("poison_damage", 10.0)
            .add_multiplier("critical", 2.0)
            .add_reduction("armor", 30.0);

        // (100 + 10 + 10) * 2.0 - 30.0 = 120 * 2.0 - 30 = 240 - 30 = 210.0
        assert_eq!(modifier.compute(), 210.0);
        assert!(modifier.has_tag("fire_damage"));
        assert!(modifier.has_tag("critical"));
        assert!(modifier.has_tag("armor"));
        assert!(!modifier.has_tag("frost"));
    }

    #[test]
    fn test_blocked_damage_returns_zero() {
        let mut modifier = CommutativeModifier::new(100.0);
        modifier.add_flat("bonus", 50.0);
        modifier.add_multiplier("quad", 4.0);
        modifier.block();

        assert_eq!(modifier.compute(), 0.0);
        assert!(modifier.is_blocked());

        modifier.unblock();
        assert_eq!(modifier.compute(), 600.0);
    }

    #[test]
    fn test_typed_blackboard_operations() {
        let mut bb = TypedBlackboard::new();
        bb.set_bool("is_headshot", true);
        bb.set_int("combo_count", 5);
        bb.set_float("distance", 350.5);
        bb.set_str("weapon", "ak47");
        bb.set_bytes("custom_meta", vec![1, 2, 3, 4]);

        assert_eq!(bb.get_bool("is_headshot"), Some(true));
        assert_eq!(bb.get_int("combo_count"), Some(5));
        assert_eq!(bb.get_float("distance"), Some(350.5));
        assert_eq!(bb.get_str("weapon"), Some("ak47"));
        assert_eq!(bb.get_bytes("custom_meta"), Some(&[1, 2, 3, 4][..]));

        assert!(bb.contains_key("weapon"));
        assert!(!bb.contains_key("missing"));

        assert_eq!(bb.remove("combo_count"), Some(BlackboardValue::Int(5)));
        assert_eq!(bb.get_int("combo_count"), None);
    }
}
