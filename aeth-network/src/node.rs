use crate::{metrics::AethMetrics, message::RollupMessage};
use std::collections::VecDeque;

/// The role this node plays in the AETH protocol.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeRole {
    Validator,
    Sequencer,
    Prover,
    Full,
}

impl std::str::FromStr for NodeRole {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "validator" => Ok(Self::Validator),
            "sequencer" => Ok(Self::Sequencer),
            "prover"    => Ok(Self::Prover),
            _           => Ok(Self::Full),
        }
    }
}

/// On-chain / in-memory state for this node.
pub struct Node {
    pub id:         [u8; 32],
    pub stake:      u64,
    pub role:       NodeRole,
    pub mempool:    VecDeque<RollupMessage>,
    pub state_root: [u8; 32],
    pub round:      u64,
    pub metrics:    AethMetrics,
}

impl Node {
    pub fn new(id: [u8; 32], stake: u64, role: NodeRole) -> Self {
        Self {
            id,
            stake,
            role,
            mempool:    VecDeque::new(),
            state_root: [0u8; 32],
            round:      0,
            metrics:    AethMetrics::new(),
        }
    }

    /// Dispatch an incoming gossip message.
    pub fn handle_message(&mut self, msg: RollupMessage) {
        match &msg {
            RollupMessage::Vote { .. } => {
                self.mempool.push_back(msg);
                self.metrics.votes_received.inc();
            }
            RollupMessage::BatchProposal { .. } => {
                tracing::debug!("Received batch proposal");
            }
            RollupMessage::Proof { state_root, .. } => {
                // TODO: plug in real verifier
                self.state_root = *state_root;
                self.metrics.proofs_generated.inc();
                tracing::info!(round = self.round, "✅ Proof accepted, state root updated");
            }
            RollupMessage::SlashReport { offender_id, reason, .. } => {
                tracing::warn!(
                    offender = offender_id.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                    reason   = reason,
                    "⚖️  Slash report received"
                );
            }
            _ => {}
        }
    }

    /// Sequencer only: gather votes into a batch proposal once threshold met.
    pub fn try_build_batch(&mut self) -> Option<RollupMessage> {
        const BATCH_THRESHOLD: usize = 32;
        if matches!(self.role, NodeRole::Sequencer | NodeRole::Full)
            && self.mempool.len() >= BATCH_THRESHOLD
        {
            let batch = RollupMessage::BatchProposal {
                round:        self.round,
                tx_hashes:    vec![], // real impl: drain mempool
                state_root:   self.state_root,
                sequencer_id: self.id,
            };
            self.mempool.clear();
            self.round += 1;
            self.metrics.batches_proposed.inc();
            tracing::info!(round = self.round - 1, "📦 Batch proposed");
            return Some(batch);
        }
        None
    }

    /// VRF-style leader election (deterministic, stake-weighted).
    pub fn is_elected_leader(&self, total_stake: u64, secret: &[u8]) -> bool {
        let input: Vec<u8> = [self.round.to_le_bytes().as_slice(), secret].concat();
        let hash  = blake3::hash(&input);
        let threshold = (u64::MAX as f64 * (self.stake as f64 / total_stake as f64)) as u64;
        u64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap()) < threshold
    }
}
