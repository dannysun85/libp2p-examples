

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

    //创建一个transport (tcp传输层 加密层也在里面 要扩展可以在此处扩展)
    let transport = libp2p::development_transport(id.clone()).await?;

    //创建网络行为协议(内置的协议 也可以自定义协议)
    let behaviour = Behaviour::new(Config::new(
        "/chat/id/1.0.0".to_string(),
        id.public(),
    ));

    //swarm框架绑定传出层和协议层
    let mut swarm = Swarm::with_threadpool_executor(transport, behaviour, nodeid);
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        match swarm.select_next_some().await{
            SwarmEvent::Behaviour(event) => println!("Behaviour Event: {:?}",event),
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
