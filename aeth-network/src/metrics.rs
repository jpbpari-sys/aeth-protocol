use std::sync::Arc;

use prometheus_client::{
    encoding::text::encode,
    metrics::{counter::Counter, gauge::Gauge},
    registry::Registry,
};

/// Shared metrics object — clone cheaply to pass around.
#[derive(Clone)]
pub struct AethMetrics {
    pub registry: Arc<Registry>,
    pub votes_received: Counter<u64>,
    pub batches_proposed: Counter<u64>,
    pub proofs_generated: Counter<u64>,
    pub forks_detected: Counter<u64>,
    /// Tracks connected peer count. Set explicitly each tick.
    pub connected_peers: Gauge,
}

impl AethMetrics {
    pub fn new() -> Self {
        let mut registry = Registry::default();

        let votes   = Counter::<u64>::default();
        let batches = Counter::<u64>::default();
        let proofs  = Counter::<u64>::default();
        let forks   = Counter::<u64>::default();
        let peers: Gauge = Gauge::default();

        registry.register("aeth_votes_received_total",   "Total votes received",      votes.clone());
        registry.register("aeth_batches_proposed_total", "Total batches proposed",    batches.clone());
        registry.register("aeth_proofs_generated_total", "Total proofs generated",    proofs.clone());
        registry.register("aeth_forks_detected_total",   "Total forks detected",      forks.clone());
        registry.register("aeth_connected_peers",         "Currently connected peers", peers.clone());

        Self {
            registry: Arc::new(registry),
            votes_received: votes,
            batches_proposed: batches,
            proofs_generated: proofs,
            forks_detected: forks,
            connected_peers: peers,
        }
    }

    /// Render metrics text for the /metrics HTTP endpoint.
    pub fn render(&self) -> String {
        let mut buf = String::new();
        encode(&mut buf, &self.registry).unwrap_or_default();
        buf
    }

    pub fn report(&self) {
        tracing::info!(
            votes     = self.votes_received.get(),
            batches   = self.batches_proposed.get(),
            proofs    = self.proofs_generated.get(),
            forks     = self.forks_detected.get(),
            peers     = self.connected_peers.get() as i64,
            "📊 AETH Metrics"
        );
    }
}

impl Default for AethMetrics {
    fn default() -> Self { Self::new() }
}
