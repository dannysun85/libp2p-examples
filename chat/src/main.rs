use std::collections::hash_map::DefaultHasher;
use std::env::args;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use libp2p::{gossipsub, mdns, noise, tcp, Transport, yamux};
use libp2p::core::transport::upgrade;
use libp2p::swarm::{NetworkBehaviour, SwarmBuilder};
use tokio::io::AsyncBufReadExt;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = false, out_event = "ChatBehaviourEvent")]
struct ChatBehaviour {
    mdns: mdns::tokio::Behaviour,
    gossip: gossipsub::Behaviour,
}

#[derive(Debug)]
enum ChatBehaviourEvent {
    Mdns(mdns::Event),
    Gossip(gossipsub::Event),
}

impl From<mdns::Event> for ChatBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        ChatBehaviourEvent::Mdns(event)
    }
}

impl From<gossipsub::Event> for ChatBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        ChatBehaviourEvent::Gossip(event)
    }
}

#[tokio::main]
async fn main() {
    let nick_name = args().nth(1);
    println!("nick_name: {:?}", nick_name);

    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = libp2p::identity::PeerId::from(local_key.public());
    println!("local_peer_id: {:?}", local_peer_id);

    let transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&id_keys).expect("signing libp2p-noise static keypair"),
        )
        .multiplex(yamux::YamuxConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let gossip_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(|message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())}) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .expect("Valid config");


    let topic = gossipsub::IdentTopic::new("chat");


    let mut swarm = {
        let mut behaviour = ChatBehaviour {
            mdns: mdns::Behaviour::new(mdns::Config::default(), local_peer_id).expect("mdns 初始化报错!"),
            gossip: gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Author(local_peer_id), gossip_config).expect("gossipsub 初始化报错!"),
        };
        behaviour.gossip.subscribe(&topic).expect("订阅主题报错!");

        SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id).build()
    };

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
}