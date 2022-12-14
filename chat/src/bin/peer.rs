use std::error::Error;
use libp2p::identify::{Behaviour, Config, Event};
use libp2p::identity::Keypair;
use libp2p::{PeerId, Swarm};
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;

#[async_std::main]
async fn main() -> Result<(),Box<dyn Error>> {

    let id = Keypair::generate_ed25519();
    let peer_id = PeerId::from(id.public());
    println!("Local peer id: {:?}", peer_id);

    // 生成一个 transport
    let transport = libp2p::development_transport(id.clone()).await?;

    let behaviour = Behaviour::new(Config::new(
        "/chat/id/1.0.0".to_string(),
        id.public()
    ));

    let mut swarm = Swarm::with_threadpool_executor(transport,behaviour,peer_id);

    if let Some(addr) = std::env::args().nth(1) {
        let remote = addr.parse()?;
        swarm.dial(remote)?;
        println!("Listening on {}", addr);
    }

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        match swarm.select_next_some().await{
            SwarmEvent::Behaviour(event) => println!("{:?}",event),
            SwarmEvent::NewListenAddr { address, .. } => println!("Listening on {:?}", address),
            SwarmEvent::Behaviour(Event::Sent { peer_id, .. }) => {
                println!("Sent identify info to {:?}", peer_id)
            }
            SwarmEvent::Behaviour(Event::Received { info, .. }) => {
                println!("Received {:?}", info)
            }
            _ => {}
        }
    }

    Ok(())
}