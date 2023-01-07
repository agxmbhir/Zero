use libp2p::{core::{
    identity::Keypair,
    transport::{MemoryTransport, Transport},
    Multiaddr
}, rendezvous::Namespace};
// Import from_str to convert a string to a Multiaddr
use std::{str::FromStr, ops::Mul};
use libp2p::futures::{prelude::*, select};
use libp2p::gossipsub::{Gossipsub, Topic, GossipsubEvent};
use libp2p::swarm::{NetworkBehaviour};
use libp2p::rendezvous;
use libp2p::gossipsub::MessageAuthenticity;

#[async_std::main]
async fn main() {

    let args: Vec<String> = std::env::args().collect();
    // Create a multiaddr from a string
    println!("Connecting to {}", args[1]);

    let addr = args[1].parse::<Multiaddr>().unwrap();

    // Generating a keypair for the node
    let localkey = Keypair::generate_ed25519();
    let local_peer_id = libp2p::PeerId::from(localkey.public());

    // Creating a development transport
    let transport = libp2p::development_transport(localkey.clone()).await.unwrap();

    // Creating a gossipsub topic
    let topic = libp2p::gossipsub::IdentTopic::new("Epoch");
    let message_authenticity = MessageAuthenticity::Signed(localkey.clone());
    let gossipsub_config = libp2p::gossipsub::GossipsubConfig::default();

    // Creating a gossipsub network behaviour
    let gossipsub: libp2p::gossipsub::Gossipsub =
        libp2p::gossipsub::Gossipsub::new(message_authenticity, gossipsub_config).unwrap();
    let rendezvous_client_behaviour = libp2p::rendezvous::client::Behaviour::new(localkey.clone());
    
    // Creating a swarm
    let mut swarm = libp2p::swarm::Swarm::with_threadpool_executor(
        transport,
         KeeperBehaviour {
            gossipsub,
            rendezvous: rendezvous_client_behaviour,
         },
         local_peer_id);
    //Litening on a port 
    swarm
        .listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap())
        .unwrap();
    
    let r_node = "/ip4/192.168.56.1/tcp/62649".parse::<Multiaddr>().unwrap();
    // Connecting to the rendezvous server
    swarm.dial(r_node).unwrap();
    // Registering the node with the rendezvous server
    swarm.behaviour_mut().rendezvous.register(Namespace::from_static("Epoch"), local_peer_id.clone(), None);
    // Discovering the rendezvous server
    // Subscribing to the "Epoch Topic"
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    loop {
        select! {
        event = swarm.select_next_some() => {
          match event {
            libp2p::swarm::SwarmEvent::Behaviour(KeeperEvent::Gossipsub(
                GossipsubEvent::Message { propagation_source, message_id, message }
            )) => {
              println!("Received message: {:?}, Source {:?}, Message ID {:?} ", message, propagation_source, message_id);
            },
            libp2p::swarm::SwarmEvent::Behaviour(KeeperEvent::Gossipsub(
                GossipsubEvent::Subscribed { peer_id, topic }
            )) => {
              println!("Peer {:?} subscribed to topic {:?}", peer_id, topic);
            },
            libp2p::swarm::SwarmEvent::Behaviour(KeeperEvent::Gossipsub(
                GossipsubEvent::Unsubscribed { peer_id, topic }
            )) => {
              println!("Peer {:?} unsubscribed from topic {:?}", peer_id, topic);
            },
            libp2p::swarm::SwarmEvent::Behaviour(KeeperEvent::Rendezvous(
                rendezvous::client::Event::Discovered
                 { rendezvous_node, registrations, cookie  }
            )) => {
                println!("Discovered rendezvous node {:?} with registrations {:?} and cookie {:?}", rendezvous_node, registrations, cookie);
                },
            _ => {println!("Received event: {:?}", event
              )}
          }
        }
        }
    }
}

// Do not implement NetworkBehaviour directly, but use the derive macro
#[derive(Debug)]
enum KeeperEvent {
    Rendezvous(rendezvous::client::Event),
    Gossipsub(GossipsubEvent),
}

impl From<rendezvous::client::Event> for KeeperEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        KeeperEvent::Rendezvous(event)
    }
}
impl From<GossipsubEvent> for KeeperEvent {
    fn from(event: GossipsubEvent) -> Self {
        KeeperEvent::Gossipsub(event)
    }
}

// Use the derive macro to create a NetworkBehaviour that automatically
// delegates to all fields that implement NetworkBehaviour.
#[derive(NetworkBehaviour)]
#[behaviour(
    out_event = "KeeperEvent",
    event_process = false,
)]
struct KeeperBehaviour {
    rendezvous: rendezvous::client::Behaviour,
    gossipsub: Gossipsub,
}
