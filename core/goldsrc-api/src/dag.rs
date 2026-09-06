//! Universal Phased Directed Acyclic Graph (PhasedDag) Ordering Engine.
//!
//! Provides a mathematically deterministic topological dependency resolver
//! with macro-phase stratification and stable tie-breaking:
//!
//! $$\text{Resolution Order} = (\text{Phase}) \longrightarrow (\text{Intra-Phase DAG}) \longrightarrow (\text{Declaration Index}) \longrightarrow (\text{Alphabetical ID})$$
//!
//! This completely eliminates manual magic priority numbers (`priority = 100, 150`)
//! in favor of intention-based semantic ordering and explicit relative anchors (`before`, `after`, `requires`).

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;

/// Contract for semantic macro-phases or architectural tiers.
///
/// Phases define a strict linear order: all nodes in an earlier phase
/// MUST be resolved before any node in a subsequent phase.
pub trait Phase: Copy + Eq + Ord + Hash + Display + Send + Sync + 'static {
    /// Static string representation of the phase.
    fn name(&self) -> &'static str;

    /// Default phase when none is explicitly specified.
    fn default_phase() -> Self;
}

/// Standard architectural layers for plugin loading.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum PluginTier {
    /// 1. Core foundation: authorization, security, basic framework infrastructure.
    Core = 10,
    /// 2. Essential background services: storage engines, database drivers, network bridges.
    Service = 20,
    /// 3. Core gameplay mechanics: game modes, VIP perks, weapon systems, economy.
    #[default]
    Gameplay = 30,
    /// 4. User interface & presentation addons: menus, HUD displays, sound effects, particles.
    Addon = 40,
    /// 5. Passive observation: telemetry, analytics, game statistics, audit loggers.
    Analytics = 50,
}

impl PluginTier {
    /// All available plugin tiers in execution order.
    pub const ALL: &'static [PluginTier] = &[
        PluginTier::Core,
        PluginTier::Service,
        PluginTier::Gameplay,
        PluginTier::Addon,
        PluginTier::Analytics,
    ];

    /// Returns static string name for the tier.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Service => "service",
            Self::Gameplay => "gameplay",
            Self::Addon => "addon",
            Self::Analytics => "analytics",
        }
    }
}

impl Display for PluginTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for PluginTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "core" => Ok(Self::Core),
            "service" => Ok(Self::Service),
            "gameplay" => Ok(Self::Gameplay),
            "addon" => Ok(Self::Addon),
            "analytics" | "monitor" => Ok(Self::Analytics),
            _ => Err(format!(
                "Unknown plugin tier: '{s}'. Expected: core, service, gameplay, addon, analytics"
            )),
        }
    }
}

impl Phase for PluginTier {
    fn name(&self) -> &'static str {
        self.as_str()
    }

    fn default_phase() -> Self {
        Self::Gameplay
    }
}

/// Semantic phases for event subscription and dispatching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum EventPhase {
    /// 1. Intercept and filter: early inspection, parameter validation, vetoing/cancellation.
    Filter = 10,
    /// 2. Primary business handling: normal execution and core responses.
    #[default]
    Handle = 20,
    /// 3. Passive observation: read-only audit logging, analytics, Discord webhooks.
    Observe = 30,
}

impl EventPhase {
    /// Static string representation of the event phase.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Filter => "filter",
            Self::Handle => "handle",
            Self::Observe => "observe",
        }
    }
}

impl Display for EventPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Phase for EventPhase {
    fn name(&self) -> &'static str {
        self.as_str()
    }

    fn default_phase() -> Self {
        Self::Handle
    }
}

/// Ordering error diagnostics returned when dependency resolution fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DagError<Id: Display> {
    /// A cyclic dependency loop was detected between nodes.
    CycleDetected { chain: String },

    /// A node required an explicitly named dependency that was not registered.
    MissingDependency { dependent: Id, required: Id },

    /// An explicit relative constraint violates linear phase ordering.
    PhaseConflict {
        from: Id,
        from_phase: &'static str,
        to: Id,
        to_phase: &'static str,
    },
}

impl<Id: Display> Display for DagError<Id> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CycleDetected { chain } => write!(f, "Cyclic dependency detected: {chain}"),
            Self::MissingDependency {
                dependent,
                required,
            } => write!(
                f,
                "Node '{dependent}' requires missing dependency '{required}'"
            ),
            Self::PhaseConflict {
                from,
                from_phase,
                to,
                to_phase,
            } => write!(
                f,
                "Phase ordering conflict: '{from}' (phase {from_phase}) cannot be ordered after '{to}' (phase {to_phase})"
            ),
        }
    }
}

impl<Id: Display + std::fmt::Debug> std::error::Error for DagError<Id> {}

/// A registered node in the phased DAG.
#[derive(Debug, Clone)]
pub struct OrderNode<P: Phase, Id, T> {
    /// Unique identifier / label for this node.
    pub id: Id,
    /// Semantic macro-phase.
    pub phase: P,
    /// Nodes that MUST run strictly after this node.
    pub before: Vec<Id>,
    /// Nodes that MUST run strictly before this node.
    pub after: Vec<Id>,
    /// Required hard dependencies (must exist in graph and run before this node).
    pub requires: Vec<Id>,
    /// Original registration index for stable tie-breaking.
    pub order_index: usize,
    /// Stored payload data.
    pub data: T,
}

/// Fluent builder for registering an individual node into `PhasedDag`.
pub struct NodeBuilder<'a, P: Phase, Id: Clone + Eq + Hash + Display + Ord, T> {
    dag: &'a mut PhasedDag<P, Id, T>,
    id: Id,
    phase: P,
    before: Vec<Id>,
    after: Vec<Id>,
    requires: Vec<Id>,
    data: T,
}

impl<'a, P: Phase, Id: Clone + Eq + Hash + Display + Ord, T> NodeBuilder<'a, P, Id, T> {
    /// Sets the semantic macro-phase of the node.
    pub fn phase(mut self, phase: P) -> Self {
        self.phase = phase;
        self
    }

    /// Declares that this node must execute before `target`.
    pub fn before(mut self, target: impl Into<Id>) -> Self {
        self.before.push(target.into());
        self
    }

    /// Declares multiple nodes that must execute after this node.
    pub fn befores(mut self, targets: impl IntoIterator<Item = Id>) -> Self {
        self.before.extend(targets);
        self
    }

    /// Declares that this node must execute after `target`.
    pub fn after(mut self, target: impl Into<Id>) -> Self {
        self.after.push(target.into());
        self
    }

    /// Declares multiple nodes that must execute before this node.
    pub fn afters(mut self, targets: impl IntoIterator<Item = Id>) -> Self {
        self.after.extend(targets);
        self
    }

    /// Declares a mandatory dependency: `target` must exist and run before this node.
    pub fn requires(mut self, target: impl Into<Id>) -> Self {
        self.requires.push(target.into());
        self
    }

    /// Declares multiple mandatory dependencies.
    pub fn requirements(mut self, targets: impl IntoIterator<Item = Id>) -> Self {
        self.requires.extend(targets);
        self
    }

    /// Commits the node to the DAG.
    pub fn register(self) {
        let order_index = self.dag.nodes.len();
        self.dag
            .label_to_indices
            .entry(self.id.clone())
            .or_default()
            .push(order_index);
        self.dag.nodes.push(OrderNode {
            id: self.id,
            phase: self.phase,
            before: self.before,
            after: self.after,
            requires: self.requires,
            order_index,
            data: self.data,
        });
    }
}

/// Phased Directed Acyclic Graph resolver.
#[derive(Debug, Clone)]
pub struct PhasedDag<P: Phase, Id: Clone + Eq + Hash + Display + Ord, T> {
    nodes: Vec<OrderNode<P, Id, T>>,
    label_to_indices: HashMap<Id, Vec<usize>>,
}

impl<P: Phase, Id: Clone + Eq + Hash + Display + Ord, T> Default for PhasedDag<P, Id, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Phase, Id: Clone + Eq + Hash + Display + Ord, T> PhasedDag<P, Id, T> {
    /// Creates a new empty `PhasedDag`.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            label_to_indices: HashMap::new(),
        }
    }

    /// Number of nodes registered in the DAG.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if no nodes are registered.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Begins registration of a new node with ID and payload data.
    pub fn add(&mut self, id: impl Into<Id>, data: T) -> NodeBuilder<'_, P, Id, T> {
        NodeBuilder {
            dag: self,
            id: id.into(),
            phase: P::default_phase(),
            before: Vec::new(),
            after: Vec::new(),
            requires: Vec::new(),
            data,
        }
    }

    /// Directly pushes an existing `OrderNode` descriptor.
    pub fn add_node(&mut self, mut node: OrderNode<P, Id, T>) {
        let order_index = self.nodes.len();
        node.order_index = order_index;
        self.label_to_indices
            .entry(node.id.clone())
            .or_default()
            .push(order_index);
        self.nodes.push(node);
    }

    /// Resolves the DAG into a deterministically ordered list of nodes.
    pub fn resolve(self) -> Result<Vec<OrderNode<P, Id, T>>, DagError<Id>> {
        // 1. Validate hard requirements and cross-phase constraints
        for node in &self.nodes {
            for req in &node.requires {
                if !self.label_to_indices.contains_key(req) {
                    return Err(DagError::MissingDependency {
                        dependent: node.id.clone(),
                        required: req.clone(),
                    });
                }
            }

            for dep_id in node.after.iter().chain(node.requires.iter()) {
                if let Some(dep_indices) = self.label_to_indices.get(dep_id) {
                    for &dep_idx in dep_indices {
                        let dep_phase = self.nodes[dep_idx].phase;
                        if node.phase < dep_phase {
                            return Err(DagError::PhaseConflict {
                                from: node.id.clone(),
                                from_phase: node.phase.name(),
                                to: dep_id.clone(),
                                to_phase: dep_phase.name(),
                            });
                        }
                    }
                }
            }

            for succ_id in &node.before {
                if let Some(succ_indices) = self.label_to_indices.get(succ_id) {
                    for &succ_idx in succ_indices {
                        let succ_phase = self.nodes[succ_idx].phase;
                        if node.phase > succ_phase {
                            return Err(DagError::PhaseConflict {
                                from: node.id.clone(),
                                from_phase: node.phase.name(),
                                to: succ_id.clone(),
                                to_phase: succ_phase.name(),
                            });
                        }
                    }
                }
            }
        }

        // 2. Identify all distinct phases in sorted order
        let mut phases: Vec<P> = self.nodes.iter().map(|n| n.phase).collect();
        phases.sort();
        phases.dedup();

        // 3. Resolve nodes phase-by-phase using Kahn's algorithm
        let mut final_resolved_indices = Vec::with_capacity(self.nodes.len());

        for current_phase in phases {
            let phase_node_indices: Vec<usize> = self
                .nodes
                .iter()
                .enumerate()
                .filter(|(_, n)| n.phase == current_phase)
                .map(|(idx, _)| idx)
                .collect();

            if phase_node_indices.is_empty() {
                continue;
            }

            // In-degree and adjacency indexed by node_idx (usize)
            let mut in_degree: HashMap<usize, usize> = HashMap::new();
            let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

            for &u_idx in &phase_node_indices {
                in_degree.insert(u_idx, 0);
                adj.entry(u_idx).or_default();
            }

            for &u_idx in &phase_node_indices {
                let node = &self.nodes[u_idx];

                // node.after(dep) => dep must run before node (dep -> node)
                for dep_id in node.after.iter().chain(node.requires.iter()) {
                    if let Some(dep_indices) = self.label_to_indices.get(dep_id) {
                        for &v_idx in dep_indices {
                            if self.nodes[v_idx].phase == current_phase && v_idx != u_idx {
                                adj.entry(v_idx).or_default().push(u_idx);
                                *in_degree.entry(u_idx).or_default() += 1;
                            }
                        }
                    }
                }

                // node.before(succ) => node must run before succ (node -> succ)
                for succ_id in &node.before {
                    if let Some(succ_indices) = self.label_to_indices.get(succ_id) {
                        for &v_idx in succ_indices {
                            if self.nodes[v_idx].phase == current_phase && v_idx != u_idx {
                                adj.entry(u_idx).or_default().push(v_idx);
                                *in_degree.entry(v_idx).or_default() += 1;
                            }
                        }
                    }
                }
            }

            // Deterministic Tie-Breaking Ready Set:
            // Order by (order_index, Id, node_idx)
            let mut ready: BTreeSet<(usize, Id, usize)> = BTreeSet::new();

            for &u_idx in &phase_node_indices {
                if in_degree[&u_idx] == 0 {
                    let order_idx = self.nodes[u_idx].order_index;
                    let id = self.nodes[u_idx].id.clone();
                    ready.insert((order_idx, id, u_idx));
                }
            }

            let mut phase_resolved = Vec::with_capacity(phase_node_indices.len());

            while let Some((_, _, curr_idx)) = ready.pop_first() {
                phase_resolved.push(curr_idx);

                if let Some(successors) = adj.get(&curr_idx) {
                    for &succ_idx in successors {
                        if let Some(deg) = in_degree.get_mut(&succ_idx) {
                            *deg -= 1;
                            if *deg == 0 {
                                let order_idx = self.nodes[succ_idx].order_index;
                                let id = self.nodes[succ_idx].id.clone();
                                ready.insert((order_idx, id, succ_idx));
                            }
                        }
                    }
                }
            }

            // Cycle detection check within phase
            if phase_resolved.len() < phase_node_indices.len() {
                let unvisited: HashSet<usize> = in_degree
                    .into_iter()
                    .filter(|(_, deg)| *deg > 0)
                    .map(|(idx, _)| idx)
                    .collect();

                let mut chain = Vec::new();
                if let Some(&start_idx) = unvisited.iter().next() {
                    let mut visited = Vec::new();
                    let mut curr = start_idx;
                    while !visited.contains(&curr) {
                        visited.push(curr);
                        if let Some(next) = adj.get(&curr).and_then(|list| {
                            list.iter().find(|idx| unvisited.contains(idx)).copied()
                        }) {
                            curr = next;
                        } else {
                            break;
                        }
                    }
                    if let Some(cycle_start_pos) = visited.iter().position(|x| *x == curr) {
                        chain = visited[cycle_start_pos..].to_vec();
                        chain.push(curr);
                    } else {
                        chain = visited;
                    }
                }

                let chain_str = chain
                    .into_iter()
                    .map(|idx| self.nodes[idx].id.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");

                return Err(DagError::CycleDetected { chain: chain_str });
            }

            final_resolved_indices.extend(phase_resolved);
        }

        // Map back indices to nodes
        let mut result = Vec::with_capacity(final_resolved_indices.len());
        let mut nodes_storage: Vec<Option<OrderNode<P, Id, T>>> =
            self.nodes.into_iter().map(Some).collect();

        for idx in final_resolved_indices {
            if let Some(node) = nodes_storage[idx].take() {
                result.push(node);
            }
        }

        Ok(result)
    }

    /// Resolves the DAG directly into an ordered list of node payloads.
    pub fn resolve_data(self) -> Result<Vec<T>, DagError<Id>> {
        self.resolve()
            .map(|nodes| nodes.into_iter().map(|n| n.data).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum TestPhase {
        First,
        Second,
        Third,
    }

    impl Display for TestPhase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::First => f.write_str("first"),
                Self::Second => f.write_str("second"),
                Self::Third => f.write_str("third"),
            }
        }
    }

    impl Phase for TestPhase {
        fn name(&self) -> &'static str {
            match self {
                Self::First => "first",
                Self::Second => "second",
                Self::Third => "third",
            }
        }

        fn default_phase() -> Self {
            Self::Second
        }
    }

    #[test]
    fn test_diamond_dag_resolution() {
        let mut dag = PhasedDag::<TestPhase, &'static str, ()>::new();

        // Diamond: A -> B, A -> C, B -> D, C -> D
        dag.add("D", ()).after("B").after("C").register();
        dag.add("B", ()).after("A").register();
        dag.add("C", ()).after("A").register();
        dag.add("A", ()).register();

        let resolved = dag.resolve().unwrap();
        let ids: Vec<&str> = resolved.iter().map(|n| n.id).collect();

        assert_eq!(ids[0], "A");
        assert_eq!(ids[3], "D");
        assert!(ids.contains(&"B") && ids.contains(&"C"));
    }

    #[test]
    fn test_phase_stratification_order() {
        let mut dag = PhasedDag::<TestPhase, &'static str, ()>::new();

        dag.add("third_item", ()).phase(TestPhase::Third).register();
        dag.add("first_item", ()).phase(TestPhase::First).register();
        dag.add("second_item", ())
            .phase(TestPhase::Second)
            .register();

        let resolved = dag.resolve().unwrap();
        let ids: Vec<&str> = resolved.iter().map(|n| n.id).collect();

        assert_eq!(ids, vec!["first_item", "second_item", "third_item"]);
    }

    #[test]
    fn test_multiple_nodes_with_same_name_preserved() {
        let mut dag = PhasedDag::<TestPhase, &'static str, usize>::new();

        dag.add("item", 1).phase(TestPhase::First).register();
        dag.add("item", 2).phase(TestPhase::Second).register();
        dag.add("item", 3).phase(TestPhase::Third).register();

        let resolved = dag.resolve().unwrap();
        let payloads: Vec<usize> = resolved.iter().map(|n| n.data).collect();

        assert_eq!(payloads, vec![1, 2, 3]);
    }

    #[test]
    fn test_cycle_detection_error() {
        let mut dag = PhasedDag::<TestPhase, &'static str, ()>::new();

        dag.add("A", ()).after("B").register();
        dag.add("B", ()).after("A").register();

        let err = dag.resolve().unwrap_err();
        match err {
            DagError::CycleDetected { chain } => {
                assert!(chain.contains("A") && chain.contains("B"));
            }
            _ => panic!("Expected CycleDetected error"),
        }
    }

    #[test]
    fn test_missing_dependency_error() {
        let mut dag = PhasedDag::<TestPhase, &'static str, ()>::new();

        dag.add("plugin_b", ())
            .requires("missing_plugin")
            .register();

        let err = dag.resolve().unwrap_err();
        assert_eq!(
            err,
            DagError::MissingDependency {
                dependent: "plugin_b",
                required: "missing_plugin"
            }
        );
    }

    #[test]
    fn test_phase_conflict_error() {
        let mut dag = PhasedDag::<TestPhase, &'static str, ()>::new();

        // Node in First phase cannot be ordered after node in Third phase
        dag.add("node_first", ())
            .phase(TestPhase::First)
            .after("node_third")
            .register();
        dag.add("node_third", ()).phase(TestPhase::Third).register();

        let err = dag.resolve().unwrap_err();
        match err {
            DagError::PhaseConflict { from, to, .. } => {
                assert_eq!(from, "node_first");
                assert_eq!(to, "node_third");
            }
            _ => panic!("Expected PhaseConflict error"),
        }
    }

    #[test]
    fn test_stable_tie_breaking_by_order_index() {
        let mut dag = PhasedDag::<TestPhase, &'static str, ()>::new();

        // All nodes have same phase and no dependencies
        dag.add("alpha", ()).register();
        dag.add("beta", ()).register();
        dag.add("gamma", ()).register();

        let resolved = dag.resolve().unwrap();
        let ids: Vec<&str> = resolved.iter().map(|n| n.id).collect();

        // Must preserve registration declaration order: alpha, beta, gamma
        assert_eq!(ids, vec!["alpha", "beta", "gamma"]);
    }
}
