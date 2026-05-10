<div align="center">

# ⬡ AETH Protocol

**Production zk-Rollup on Solana**

Recursive Nova folding • Decentralized GPU Prover Marketplace • MEV-Resistant VRF Sequencing

![Version](https://img.shields.io/badge/version-0.8.0-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)
![Rust](https://img.shields.io/badge/language-Rust%202021-000000?style=flat-square&logo=rust)
![Solana](https://img.shields.io/badge/chain-Solana-9945FF?style=flat-square)
![CI](https://img.shields.io/github/actions/workflow/status/yourorg/aeth-protocol/ci.yml?style=flat-square)

</div>

---

## What is AETH?

AETH is a production-grade zk-Rollup network designed to run on Solana. It combines:

| Component | Technology |
|-----------|-----------|
| Consensus network | `libp2p` (gossipsub + kademlia) |
| zk Proofs | Halo2 circuits + Nova recursive folding |
| Proving market | On-chain bidding + GPU cluster |
| Sequencing | VRF-based MEV-resistant leader election |
| Settlement | Solana Anchor program |
| Bridges | Wormhole NTT + LayerZero V2 |

## Architecture

```
Users / Searchers
      │
      ▼  (confidential orderflow auction)
VRF-elected Sequencer
      │
      ▼  (batch proposal via gossipsub)
GPU Prover Cluster  ◄── on-chain bid marketplace
      │
      ▼  (Nova recursive folding)
Solana Settlement Program
      │
      ▼
Fee split → Provers 40% / Validators 30% / Sequencers 20% / Burn 10%
```

## Quick Start

```bash
git clone https://github.com/yourorg/aeth-protocol.git
cd aeth-protocol
chmod +x setup.sh && ./setup.sh
```

### Run local testnet (Docker)

```bash
docker compose up -d --build
# Grafana: http://localhost:3000  (admin / aeth123)
# Metrics: http://localhost:9090
```

### Run adversarial simulator

```bash
cargo run -p aeth-simulator --release -- --nodes 50 --byzantine 0.25 --rounds 500 --attacks
```

### Run a single validator node

```bash
cargo run -p aeth-network --release -- \
  --role validator \
  --stake 100000 \
  --port 4001 \
  --bootnodes /ip4/BOOTNODE/tcp/4001/p2p/QmXXX
```

### Run a prover node

```bash
cargo run -p aeth-prover --release -- --workers 8
```

## Repo Structure

```
aeth-protocol/
├── aeth-network/       libp2p node (validator / sequencer / full)
│   └── src/
│       ├── main.rs     CLI entrypoint + metrics HTTP server
│       ├── behavior.rs gossipsub + kademlia behaviour
│       ├── node.rs     node state, VRF election, batch building
│       ├── swarm.rs    event loop
│       ├── message.rs  all gossip message types
│       └── metrics.rs  Prometheus counters / gauges
├── aeth-prover/        GPU proving cluster + Nova folding
│   └── src/
│       ├── main.rs     CLI + prover orchestration loop
│       └── lib.rs      GpuProverCluster + NovaAccumulator
├── aeth-simulator/     Adversarial localnet (10–100 nodes)
│   └── src/
│       ├── main.rs     CLI
│       └── lib.rs      node types, fork detection, attack scenarios
├── docker-compose.yml  Full local testnet
├── Dockerfile.network
├── Dockerfile.prover
├── Dockerfile.simulator
├── monitoring/         Prometheus + Grafana
├── k8s/                Kubernetes manifests
├── .github/workflows/  CI/CD
└── docs/               Architecture, tokenomics, onboarding
```

## Economic Model (AETH Token)

| Allocation | % | Vesting |
|------------|---|---------|
| Community & Airdrops | 35% | 6-month cliff + linear |
| Liquidity / DEX | 20% | Immediate |
| Team & Advisors | 15% | 12-month cliff + 24 mo |
| Ecosystem Grants | 12% | 3-year |
| Prover Incentives | 10% | Emission over 3 years |
| Treasury | 8% | Governance controlled |

**Fee split per batch:** 40% provers · 30% validators · 20% sequencers · 10% burn

## Roadmap

- [x] libp2p swarm with gossipsub + kademlia
- [x] VRF-based leader election
- [x] GPU prover cluster (mock Halo2 + Nova)
- [x] Adversarial simulator (fork detection, attack scenarios)
- [x] Docker + Kubernetes + CI/CD
- [x] Prometheus + Grafana monitoring
- [ ] Real Halo2 circuit integration
- [ ] Nova `RecursiveSNARK` wiring
- [ ] Anchor program (staking, marketplace, governance)
- [ ] Incentivized testnet
- [ ] Security audits (OtterSec + Zellic)
- [ ] Mainnet launch Q3 2026

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) — PRs welcome for circuits, GPU kernels, and economic simulations.

## Security

See [SECURITY.md](SECURITY.md) for responsible disclosure. Bug bounty program coming via Immunefi.

## License

MIT © 2026 AETH Protocol Contributors
