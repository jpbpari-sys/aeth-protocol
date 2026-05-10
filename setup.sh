#!/usr/bin/env bash
# setup.sh — one-command bootstrap for AETH Protocol
set -e

echo "🚀 AETH Protocol Setup"
echo "======================"

# ── Rust ──────────────────────────────────────────────────────────────────────
if ! command -v rustc &>/dev/null; then
    echo "🦀 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi
echo "✅ Rust $(rustc --version)"

# ── Build ─────────────────────────────────────────────────────────────────────
echo ""
echo "🔨 Building AETH workspace..."
cargo build --workspace --release

echo ""
echo "🧪 Running tests..."
cargo test --workspace

# ── .env ──────────────────────────────────────────────────────────────────────
if [ ! -f .env ]; then
    cp .env.example .env
    echo "📄 Created .env from .env.example — edit before running"
fi

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║  ✅ AETH Protocol ready to launch!           ║"
echo "╠══════════════════════════════════════════════╣"
echo "║  Local testnet:                              ║"
echo "║    docker compose up -d --build              ║"
echo "║                                              ║"
echo "║  Adversarial simulator:                      ║"
echo "║    cargo run -p aeth-simulator --release     ║"
echo "║                 -- --attacks                 ║"
echo "║                                              ║"
echo "║  Single validator node:                      ║"
echo "║    cargo run -p aeth-network --release       ║"
echo "║                 -- --role validator          ║"
echo "╚══════════════════════════════════════════════╝"
