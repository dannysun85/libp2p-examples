
use libp2p::{identity, PeerId};

fn main() {
    let id = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id.public());
}