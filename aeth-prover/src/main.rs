use aeth_prover::{GpuProverCluster, NovaAccumulator, ProverTask};
use axum::{routing::get, Router};
use clap::Parser;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about = "AETH Protocol — GPU Prover Node")]
struct Args {
    /// Number of parallel proving workers
    #[arg(long, default_value_t = 8)]
    workers: usize,

    /// Metrics / health HTTP port
    #[arg(long, default_value_t = 8083)]
    metrics_port: u16,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,aeth_prover=debug".into()),
        )
        .init();

    tracing::info!("🔥 AETH Prover starting | Workers: {}", args.workers);

    // Channel: workers → accumulator
    let (result_tx, mut result_rx) = mpsc::channel::<(u64, Vec<u8>)>(256);

    let cluster = GpuProverCluster::new(args.workers, result_tx);
    cluster.start();

    // Metrics endpoint
    let metrics_port = args.metrics_port;
    tokio::spawn(async move {
        let app = Router::new()
            .route("/health",  get(|| async { "OK" }))
            .route("/metrics", get(|| async { "aeth_prover_up 1\n" }));
        let addr = format!("0.0.0.0:{metrics_port}");
        tracing::info!("📊 Metrics at http://{addr}/metrics");
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Submit demo tasks so the prover actually does something on start
    for i in 0..5u64 {
        cluster.submit(ProverTask {
            batch_id: i,
            witness:  vec![i as u8; 128 + (i * 32) as usize],
            priority: 1,
        });
    }

    // Aggregate results via Nova folding
    let mut nova = NovaAccumulator::new();
    tracing::info!("Waiting for proofs…");

    while let Some((batch_id, partial_proof)) = result_rx.recv().await {
        let recursive = nova.fold(&partial_proof);
        let valid      = nova.verify(&recursive);
        tracing::info!(
            batch  = batch_id,
            folded = nova.folded_rounds,
            valid  = valid,
            "✅ Proof folded | recursive proof {} bytes",
            recursive.len()
        );
    }

    Ok(())
}
