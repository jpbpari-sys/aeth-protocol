use libp2p::{
    gossipsub::{self, IdentTopic},
    identify,
    kad::{self, store::MemoryStore},
    swarm::NetworkBehaviour,
    Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use std::time::Duration;
use crate::message::RollupMessage;

/// The combined libp2p behaviour for AETH nodes.
#[derive(NetworkBehaviour)]
pub struct AethBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub identify:  identify::Behaviour,
    pub kademlia:  kad::Behaviour<MemoryStore>,
}

/// Build a fully configured libp2p Swarm.
pub async fn build_swarm(
    keypair:     libp2p::identity::Keypair,
    listen_addr: Multiaddr,
) -> anyhow::Result<Swarm<AethBehaviour>> {
    let peer_id = PeerId::from(keypair.public());

    let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // Gossipsub
            let gs_cfg = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(5))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .build()
                .expect("valid gossipsub config");
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gs_cfg,
            )
            .expect("gossipsub init");

            // Identify
            let identify = identify::Behaviour::new(identify::Config::new(
                "/aeth/1.0.0".to_string(),
                key.public(),
            ));

            // Kademlia
            let kademlia = kad::Behaviour::new(peer_id, MemoryStore::new(peer_id));

            AethBehaviour { gossipsub, identify, kademlia }
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.listen_on(listen_addr)?;
    Ok(swarm)
}

/// Topic used for all AETH gossip messages.
pub fn aeth_topic() -> IdentTopic {
    IdentTopic::new("aeth-rollup-v1")
}

/// Publish a RollupMessage to the gossip mesh.
pub fn broadcast(swarm: &mut Swarm<AethBehaviour>, msg: &RollupMessage) -> anyhow::Result<()> {
    let data = bincode::serialize(msg)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(aeth_topic(), data)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("gossipsub publish error: {e}"))
}
