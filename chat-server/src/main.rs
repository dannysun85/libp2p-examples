use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use libp2p::{gossipsub, identity, mdns, PeerId, Swarm, swarm};
use libp2p::gossipsub::{Gossipsub, GossipsubMessage, MessageAuthenticity, MessageId, Topic};

#[async_std::main]
async fn main() -> Result<(),Box<dyn Error>> {

    let id = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id.public());
    println!("Peer id {:?}", peer_id);

    let transport = libp2p::development_transport(id.clone()).await?;

    let topic = Topic::new("chat");

    #[derive(NetworkBehaviour)]
    struct MyBehaviour{
        gossipsub:Gossipsub,
        mdns:mdns::async_io::Behaviour,
    }

    //设置message的加密内容
    let mut swarm = {
        let message_id_fn = |message:&GossipsubMessage|{
            let mut hasher = DefaultHasher::new();
            message.data.hash(&mut hasher);
            MessageId::from(hasher.finish().to_string())
        };

        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(message_id_fn)
            .build()
            .expect("valid config");

        let mut gossipsub:gossipsub::Gossipsub = gossipsub::Gossipsub::new(MessageAuthenticity::Signed(id), gossipsub_config).expect("Correct configuration");

        gossipsub.subscribe(&topic.clone()).unwrap();

        if let Some(explicit) = std::env::args().nth(2){
            let explicit = explicit.clone();
            match explicit.parse(){
                Ok(id) => gossipsub.add_explicit_peer(&id),
                Err(err) => println!("Failed to parse explicit peer id: {:?}", err),
            }
        }

        Swarm::with_threadpool_executor(transport, gossipsub, peer_id);

    };

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();



    Ok(())
}

// pub fn message_name(name:&str,message:&GossipsubMessage) -> String{
//     let mut name_map = HashMap::new();
//     name_map.insert(name,message.source.clone());
//}