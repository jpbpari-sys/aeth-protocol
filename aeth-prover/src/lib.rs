use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tracing::{info, warn};

/// A unit of proving work dispatched to a worker thread.
#[derive(Debug, Clone)]
pub struct ProverTask {
    pub batch_id:  u64,
    pub witness:   Vec<u8>,
    pub priority:  u32,
}

/// Completed proof output: (batch_id, compressed_proof_bytes).
pub type ProofResult = (u64, Vec<u8>);

/// A pool of worker threads that each pick tasks and generate proofs.
#[derive(Clone)]
pub struct GpuProverCluster {
    pub num_workers: usize,
    queue: Arc<Mutex<Vec<ProverTask>>>,
    result_tx: mpsc::Sender<ProofResult>,
}

impl GpuProverCluster {
    pub fn new(num_workers: usize, result_tx: mpsc::Sender<ProofResult>) -> Self {
        Self {
            num_workers,
            queue: Arc::new(Mutex::new(Vec::new())),
            result_tx,
        }
    }

    /// Submit a task to the proving queue.
    pub fn submit(&self, task: ProverTask) {
        self.queue.lock().unwrap().push(task);
    }

    /// Spawn background worker threads (call once at startup).
    pub fn start(&self) {
        for worker_id in 0..self.num_workers {
            let queue  = self.queue.clone();
            let result = self.result_tx.clone();

            std::thread::spawn(move || {
                info!("🔧 Prover worker {worker_id} started");
                loop {
                    let task = {
                        let mut q = queue.lock().unwrap();
                        if q.is_empty() {
                            drop(q);
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        q.remove(0)
                    };

                    info!(batch = task.batch_id, worker = worker_id, "⚙️  Proving batch");
                    let proof = Self::prove(&task.witness);

                    if result.try_send((task.batch_id, proof)).is_err() {
                        warn!("result channel full – dropping proof for batch {}", task.batch_id);
                    }
                }
            });
        }
    }

    /// Simulate Halo2 / Nova proving (replace with real circuit call).
    ///
    /// Real integration:
    ///   1. Transfer witness to GPU via cudarc
    ///   2. Run MSM + NTT kernels
    ///   3. Call halo2_proofs::plonk::create_proof
    fn prove(witness: &[u8]) -> Vec<u8> {
        // Simulate heavy GPU computation
        let delay_ms = 50 + (witness.len() as u64 % 250);
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));

        // Return mock 512-byte compressed proof (replace with real output)
        let mut proof = vec![0xAAu8; 512];
        proof[0..8].copy_from_slice(&(witness.len() as u64).to_le_bytes());
        proof
    }
}

/// Nova recursive folding accumulator (mock layer).
///
/// Real integration: use `nova_snark::RecursiveSNARK` to fold partial proofs
/// into a single logarithmic-size recursive proof.
pub struct NovaAccumulator {
    pub folded_rounds: u64,
    pub state:         Vec<u8>,
}

impl NovaAccumulator {
    pub fn new() -> Self {
        Self { folded_rounds: 0, state: vec![0u8; 32] }
    }

    /// Fold a new partial proof into the accumulator.
    pub fn fold(&mut self, partial_proof: &[u8]) -> Vec<u8> {
        // Mock: XOR state with first 32 bytes of partial proof
        for (i, &b) in partial_proof.iter().take(32).enumerate() {
            self.state[i] ^= b;
        }
        self.folded_rounds += 1;

        // Return "compressed" recursive proof
        let mut out = vec![0u8; 256];
        out[0..8].copy_from_slice(&self.folded_rounds.to_le_bytes());
        out[8..40].copy_from_slice(&self.state);
        out
    }

    pub fn verify(&self, proof: &[u8]) -> bool {
        proof.len() >= 8 // minimal sanity check; replace with real verifier
    }
}

impl Default for NovaAccumulator {
    fn default() -> Self { Self::new() }
}
