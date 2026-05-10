use rand::Rng;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn};

// ─── Node types ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum NodeType {
    Honest,
    Slow,
    Byzantine,
    LazySequencer,
}

#[derive(Clone, Debug)]
pub struct SimNode {
    pub id:          usize,
    pub node_type:   NodeType,
    pub state_root:  [u8; 32],
    pub round:       u64,
    pub stake:       u64,
    pub slash_count: u32,
    pub latency_ms:  u64,
}

impl SimNode {
    pub fn new(id: usize, node_type: NodeType, stake: u64) -> Self {
        let latency_ms = match node_type {
            NodeType::Slow => rand::thread_rng().gen_range(200..600),
            _              => rand::thread_rng().gen_range(10..80),
        };
        Self {
            id,
            node_type,
            state_root: [0u8; 32],
            round: 0,
            stake,
            slash_count: 0,
            latency_ms,
        }
    }

    /// Honest vote: XOR id into root deterministically.
    pub fn cast_vote(&mut self, round: u64) {
        self.round = round;
        self.state_root[0] ^= self.id as u8;
    }

    /// Byzantine: emit a random conflicting state root.
    pub fn cast_conflicting_vote(&mut self, round: u64) {
        self.round = round;
        let mut rng = rand::thread_rng();
        rng.fill(&mut self.state_root);
        warn!(id = self.id, round, "⚠️  Byzantine conflicting vote");
    }
}

// ─── Fork detection ───────────────────────────────────────────────────────────

pub fn detect_forks(nodes: &[SimNode]) -> bool {
    let roots: HashSet<[u8; 32]> = nodes.iter().map(|n| n.state_root).collect();
    if roots.len() > 1 {
        warn!(unique_roots = roots.len(), "🔱 Fork detected!");
        return true;
    }
    false
}

// ─── Attack scenarios ─────────────────────────────────────────────────────────

pub enum AttackType {
    Collusion33,
    ProverCartel,
    SlashingGrief,
}

pub async fn run_adversarial_scenario(nodes: &mut Vec<SimNode>, attack: AttackType, rounds: u64) {
    let mut rng = rand::thread_rng();

    for round in 0..rounds {
        match attack {
            AttackType::Collusion33 => {
                for node in nodes.iter_mut() {
                    if node.id % 3 == 0 {
                        node.cast_conflicting_vote(round);
                    } else {
                        node.cast_vote(round);
                    }
                }
            }
            AttackType::ProverCartel => {
                // Top 3 nodes "slow-proof" with 40% probability
                for node in nodes.iter_mut().take(3) {
                    if rng.gen_bool(0.4) {
                        sleep(Duration::from_millis(500)).await;
                    }
                    node.cast_vote(round);
                }
                for node in nodes.iter_mut().skip(3) {
                    node.cast_vote(round);
                }
            }
            AttackType::SlashingGrief => {
                // Submit invalid slash reports against honest nodes
                for node in nodes.iter_mut().take(5) {
                    node.slash_count += 1;
                    warn!(id = node.id, "🔪 Grief slash attempt");
                }
                for node in nodes.iter_mut() {
                    node.cast_vote(round);
                }
            }
        }

        if detect_forks(nodes) {
            warn!(round, "🚨 Fork survived round {round}");
        }
        sleep(Duration::from_millis(20)).await;
    }
}

// ─── Main simulation ─────────────────────────────────────────────────────────

pub struct SimConfig {
    pub num_nodes:       usize,
    pub byzantine_ratio: f64,
    pub rounds:          u64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self { num_nodes: 50, byzantine_ratio: 0.25, rounds: 500 }
    }
}

pub fn create_nodes(config: &SimConfig) -> Vec<SimNode> {
    let byz_count = (config.num_nodes as f64 * config.byzantine_ratio) as usize;
    let mut rng   = rand::thread_rng();

    (0..config.num_nodes)
        .map(|id| {
            let node_type = if id < byz_count {
                if rng.gen_bool(0.5) { NodeType::Byzantine } else { NodeType::Slow }
            } else {
                NodeType::Honest
            };
            SimNode::new(id, node_type, rng.gen_range(10_000..=500_000))
        })
        .collect()
}

pub async fn run_simulation(config: SimConfig) {
    let mut nodes = create_nodes(&config);

    info!(
        total        = nodes.len(),
        byzantine    = (nodes.len() as f64 * config.byzantine_ratio) as usize,
        rounds       = config.rounds,
        "🧪 Simulation started"
    );

    let mut fork_count = 0u64;

    for round in 0..config.rounds {
        for node in nodes.iter_mut() {
            match node.node_type {
                NodeType::Byzantine    => node.cast_conflicting_vote(round),
                NodeType::Honest       => node.cast_vote(round),
                NodeType::Slow         => {
                    sleep(Duration::from_millis(node.latency_ms)).await;
                    node.cast_vote(round);
                }
                NodeType::LazySequencer => {}
            }
        }

        if detect_forks(&nodes) {
            fork_count += 1;
        }

        // Re-align honest nodes to majority root (simplified BFT convergence)
        let majority_root = {
            let roots: Vec<[u8; 32]> = nodes
                .iter()
                .filter(|n| matches!(n.node_type, NodeType::Honest))
                .map(|n| n.state_root)
                .collect();
            roots.first().copied().unwrap_or([0u8; 32])
        };
        for n in nodes.iter_mut() {
            if matches!(n.node_type, NodeType::Honest) {
                n.state_root = majority_root;
            }
        }

        if round % 100 == 0 {
            info!(round, forks = fork_count, "📊 Sim progress");
        }

        sleep(Duration::from_millis(5)).await;
    }

    let fork_rate = fork_count as f64 / config.rounds as f64;
    info!(
        fork_count,
        fork_rate = format!("{:.2}%", fork_rate * 100.0),
        "✅ Simulation complete"
    );

    if fork_rate < 0.01 {
        info!("🟢 System STABLE under adversarial load");
    } else {
        warn!("🔴 System UNSTABLE — tune slashing or increase honest ratio");
    }
}
