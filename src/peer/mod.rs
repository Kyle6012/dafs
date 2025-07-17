use libp2p::{
    identity,
    mdns::tokio::Behaviour as MdnsBehaviour,
    swarm::{Swarm, SwarmEvent},
    PeerId,
    noise,
    tcp::tokio::Transport as TokioTcpTransport,
    core::upgrade,
    yamux,
    Transport,
};
use anyhow::Result;
use futures::StreamExt;

#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct MyBehaviour {
    pub mdns: MdnsBehaviour,
}

pub async fn start_node() -> Result<()> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());
    println!("Local peer id: {:?}", peer_id);

    // Build transport manually
    let transport = TokioTcpTransport::new(
        libp2p::tcp::Config::default().nodelay(true),
    )
    .upgrade(upgrade::Version::V1)
    .authenticate(noise::Config::new(&id_keys)?)
    .multiplex(yamux::Config::default())
    .boxed();

    let behaviour = MyBehaviour {
        mdns: MdnsBehaviour::new(Default::default(), peer_id)?,
    };
    let mut swarm = Swarm::new(transport, behaviour, peer_id, libp2p::swarm::Config::with_tokio_executor());
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                println!("Swarm event: {:?}", event);
            }
        }
    }
}
