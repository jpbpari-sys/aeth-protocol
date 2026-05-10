use aeth_simulator::{run_adversarial_scenario, run_simulation, AttackType, SimConfig};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "AETH Protocol — Adversarial Localnet Simulator")]
struct Args {
    /// Number of nodes to simulate
    #[arg(long, default_value_t = 50)]
    nodes: usize,

    /// Ratio of Byzantine nodes (0.0–0.49)
    #[arg(long, default_value_t = 0.25)]
    byzantine: f64,

    /// Number of rounds per scenario
    #[arg(long, default_value_t = 500)]
    rounds: u64,

    /// Also run adversarial attack scenarios
    #[arg(long)]
    attacks: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,aeth_simulator=debug".into()),
        )
        .init();

    tracing::info!(
        "🚀 AETH Simulator | nodes={} | byzantine={:.0}% | rounds={}",
        args.nodes, args.byzantine * 100.0, args.rounds
    );

    let config = SimConfig {
        num_nodes:       args.nodes,
        byzantine_ratio: args.byzantine,
        rounds:          args.rounds,
    };

    run_simulation(config).await;

    if args.attacks {
        tracing::info!("--- Running: Collusion33 ---");
        let mut nodes = aeth_simulator::create_nodes(&SimConfig {
            num_nodes: args.nodes, byzantine_ratio: args.byzantine, rounds: args.rounds,
        });
        run_adversarial_scenario(&mut nodes, AttackType::Collusion33, args.rounds).await;

        tracing::info!("--- Running: ProverCartel ---");
        let mut nodes2 = aeth_simulator::create_nodes(&SimConfig {
            num_nodes: args.nodes, byzantine_ratio: args.byzantine, rounds: args.rounds,
        });
        run_adversarial_scenario(&mut nodes2, AttackType::ProverCartel, args.rounds).await;
    }

    Ok(())
}
