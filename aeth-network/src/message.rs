use serde::{Deserialize, Serialize};

/// All messages gossiped across the AETH p2p network.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RollupMessage {
    Vote {
        round: u64,
        proposal_hash: [u8; 32],
        signature: Vec<u8>,       // Ed25519 / BLS sig bytes (64+)
        validator_id: [u8; 32],
    },
    BatchProposal {
        round: u64,
        tx_hashes: Vec<[u8; 32]>,
        state_root: [u8; 32],
        sequencer_id: [u8; 32],
    },
    Proof {
        round: u64,
        proof_bytes: Vec<u8>,
        state_root: [u8; 32],
        public_inputs: Vec<u8>,
    },
    Heartbeat {
        node_id: Vec<u8>, // PeerId bytes
        stake: u64,
        timestamp: u64,
        round: u64,
    },
    SlashReport {
        offender_id: [u8; 32],
        reason: String,
        evidence: Vec<u8>,
    },
}

/// A single user transaction inside a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AethTransaction {
    pub nonce: u64,
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub amount: u64,
    pub fee: u64,
}
