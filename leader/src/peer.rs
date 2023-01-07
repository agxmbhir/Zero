use libp2p::{PeerId, identity};

pub struct Peer {
    pub key: identity::Keypair,
    pub address: PeerId,
}
// Implementing Peer
impl Peer {
    pub fn new(key: identity::Keypair, address: PeerId) -> Self {
        Peer { key, address }
    }
    // Create a new Peer with a random PeerId
    pub fn init_random() -> Peer {
        // Create a random PeerId
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        Peer {
            key: local_key,
            address: local_peer_id,
        }
    }
}
