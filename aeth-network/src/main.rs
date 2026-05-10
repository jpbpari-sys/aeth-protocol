use aeth_network::{build_swarm, Node, NodeRole};
use axum::{routing::get, Router};
use clap::Parser;
use libp2p::Multiaddr;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about = "AETH Protocol — Network Node")]
struct Args {
    /// Role: validator | sequencer | prover | full
    #[arg(long, default_value = "full")]
    role: String,

    /// Stake (in AETH smallest unit)
    #[arg(long, default_value_t = 100_000)]
    stake: u64,

    /// libp2p listen port
    #[arg(long, default_value_t = 4001)]
    port: u16,

    /// Prometheus metrics HTTP port
    #[arg(long, default_value_t = 8080)]
    metrics_port: u16,

    /// Bootnode multiaddrs to dial on startup (comma-separated)
    #[arg(long, value_delimiter = ',', default_values_t = Vec::<String>::new())]
    bootnodes: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,aeth_network=debug".into()),
        )
        .init();

    let keypair  = libp2p::identity::Keypair::generate_ed25519();
    let peer_id  = libp2p::PeerId::from(keypair.public());
    let listen: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", args.port).parse()?;

    tracing::info!("🚀 AETH Node starting | PeerId: {peer_id} | Role: {} | Port: {}", args.role, args.port);

    let mut swarm = build_swarm(keypair.clone(), listen).await?;

    // Dial bootnodes
    for addr_str in &args.bootnodes {
        if let Ok(addr) = Multiaddr::from_str(addr_str) {
            let _ = swarm.dial(addr);
        }
    }

    let role: NodeRole = args.role.parse()?;
    let node_id = {
        let bytes = peer_id.to_bytes();
        let mut id = [0u8; 32];
        let len = bytes.len().min(32);
        id[..len].copy_from_slice(&bytes[..len]);
        id
    };
    let node = Node::new(node_id, args.stake, role);

    // Start metrics HTTP server
    let metrics_clone = node.metrics.clone();
    tokio::spawn(async move {
        let app = Router::new()
            .route("/metrics", get(move || {
                let m = metrics_clone.clone();
                async move { m.render() }
            }))
            .route("/health", get(|| async { "OK" }));

        let addr = format!("0.0.0.0:{}", args.metrics_port);
        tracing::info!("📊 Metrics at http://{addr}/metrics");
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Main event loop
    aeth_network::run_node(swarm, node).await?;

    Ok(())
}
