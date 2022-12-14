

use std::error::Error;
use libp2p::{identity, PeerId, Swarm};
use libp2p::futures::StreamExt;
use libp2p::identify::{Config, Event, Behaviour};
use libp2p::swarm::SwarmEvent;

#[async_std::main]
async fn main() -> Result<(),Box<dyn Error>> {

    // 创建随机id
    let id = identity::Keypair::generate_ed25519();
    let nodeid = PeerId::from(id.public());
    println!("nodeid : {:?}",nodeid);

    //创建一个transport
    let transport = libp2p::development_transport(id.clone()).await?;

    //创建网络行为协议
    let behaviour = Behaviour::new(Config::new(
        "/chat/id/1.0.0".to_string(),
        id.public(),
    ));

    let mut swarm = Swarm::with_threadpool_executor(transport, behaviour, nodeid);
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
