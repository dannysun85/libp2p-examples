use std::collections::hash_map::DefaultHasher;
use std::env::args;
use std::error::Error;
use std::hash::{Hash, Hasher};
use tokio::io::AsyncBufReadExt;
use std::time::Duration;
use futures::{prelude::*, select};
use libp2p::{gossipsub, mdns, noise, tcp, Transport, yamux};
use libp2p::core::transport::upgrade;
use libp2p::swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent};
use tokio::io;

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
async fn main() -> Result<(), Box<dyn Error>> {
    let nick_name = args().nth(1);
    println!("nick_name: {:?}", nick_name);

    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = libp2p::identity::PeerId::from(local_key.public());
    println!("local_peer_id: {:?}", local_peer_id);

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&local_key).expect("signing libp2p-noise static keypair"),
        )
        .multiplex(yamux::YamuxConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let gossip_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
        .message_id_fn(move |message: &gossipsub::Message| {
            if !nick_name.is_none() {
                return gossipsub::MessageId::from(&*message.data);
            }else {
                let mut hasher = DefaultHasher::new();
                nick_name.hash(&mut hasher);
                gossipsub::MessageId::from(hasher.finish().to_string())
            }
        }) // content-address messages. No two messages of the same content will be propagated.
        .build()
        .expect("Valid config");


    let topic = gossipsub::IdentTopic::new("chat");


    let mut swarm = {
        let mut behaviour = ChatBehaviour {
            mdns: mdns::Behaviour::new(mdns::Config::default(), local_peer_id.clone()).expect("mdns 初始化报错!"),
            gossip: gossipsub::Behaviour::new(gossipsub::MessageAuthenticity::Signed(local_key.clone()), gossip_config).expect("gossipsub 初始化报错!"),
        };

        behaviour.gossip.subscribe(&topic).expect("订阅主题报错!");

        SwarmBuilder::with_tokio_executor(transport, behaviour, local_peer_id.clone()).build()
    };

    // 从标准输入中读取消息
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    loop {
        select! {
            line = stdin.next_line().fuse() => {
                if let Err(e) = swarm.behaviour_mut().gossip.publish(topic.clone(), line.expect("Stdin not to close").expect(" 这个错误后面添加的").as_bytes()) {
                    println!("Publish error: {e:?}");
                }
            },
            event = swarm.select_next_some()=> match event {
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossip.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossip.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(ChatBehaviourEvent::Gossip(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    ),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                }
                _ => {}
            }
        }
    }
}