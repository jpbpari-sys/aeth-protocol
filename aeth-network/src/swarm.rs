use crate::{
    behavior::{aeth_topic, broadcast, AethBehaviour},
    message::RollupMessage,
    node::Node,
};
use futures::StreamExt as _;
use libp2p::{gossipsub, swarm::SwarmEvent, Swarm};
use std::time::Duration;
use tokio::time;

/// Main event-loop: drives the swarm, sends heartbeats, proposes batches.
pub async fn run_node(
    mut swarm: Swarm<AethBehaviour>,
    mut node: Node,
) -> anyhow::Result<()> {
    // Subscribe to the AETH gossip topic.
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&aeth_topic())
        .expect("subscribe to aeth topic");

    let mut heartbeat_ticker = time::interval(Duration::from_secs(10));
    let mut batch_ticker     = time::interval(Duration::from_secs(30));
    let mut metrics_ticker   = time::interval(Duration::from_secs(60));

    tracing::info!(
        role  = ?node.role,
        stake = node.stake,
        "🚀 AETH node event loop started"
    );

    loop {
        tokio::select! {
            // ── libp2p events ─────────────────────────────────────────────
            event = swarm.select_next_some() => {
                handle_event(&mut swarm, &mut node, event);
            }

            // ── Heartbeat broadcast ────────────────────────────────────────
            _ = heartbeat_ticker.tick() => {
                let hb = RollupMessage::Heartbeat {
                    node_id:   node.id.to_vec(),
                    stake:     node.stake,
                    timestamp: chrono::Utc::now().timestamp() as u64,
                    round:     node.round,
                };
                if let Err(e) = broadcast(&mut swarm, &hb) {
                    tracing::warn!("heartbeat broadcast failed: {e}");
                }
                let peers = swarm.connected_peers().count() as i64;
                node.metrics.connected_peers.set(peers);
            }

            // ── Batch building (sequencer) ─────────────────────────────────
            _ = batch_ticker.tick() => {
                if let Some(batch) = node.try_build_batch() {
                    if let Err(e) = broadcast(&mut swarm, &batch) {
                        tracing::warn!("batch broadcast failed: {e}");
                    }
                }
            }

            // ── Metrics dump ───────────────────────────────────────────────
            _ = metrics_ticker.tick() => {
                node.metrics.report();
            }
        }
    }
}

/// Dispatch a single swarm event.
fn handle_event(
    _swarm: &mut Swarm<AethBehaviour>,
    node: &mut Node,
    event: SwarmEvent<crate::behavior::AethBehaviourEvent>,
) {
    use libp2p::swarm::SwarmEvent::*;

    match event {
        Behaviour(crate::behavior::AethBehaviourEvent::Gossipsub(
            gossipsub::Event::Message { message, .. },
        )) => {
            match bincode::deserialize::<RollupMessage>(&message.data) {
                Ok(msg) => node.handle_message(msg),
                Err(e)  => tracing::debug!("failed to deserialise gossip msg: {e}"),
            }
        }

        NewListenAddr { address, .. } => {
            tracing::info!("Listening on {address}");
        }
        ConnectionEstablished { peer_id, .. } => {
            tracing::info!("Connected to {peer_id}");
        }
        ConnectionClosed { peer_id, .. } => {
            tracing::info!("Disconnected from {peer_id}");
        }
        OutgoingConnectionError { error, .. } => {
            tracing::warn!("Outgoing connection error: {error}");
        }
        _ => {}
    }
}
