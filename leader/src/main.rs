mod peer;
use peer::{Peer};
use futures::{prelude::*};
use libp2p::
{
    gossipsub::{Gossipsub, GossipsubEvent, MessageAuthenticity},
    rendezvous,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
};
use tokio::{self, select };
// Import Arc from tokio
// BufRead
use async_std::{ io };
#[tokio::main]
async fn main() {

    let peer = Peer::init_random();
    
    // Selecting a transport, not using a transport layer security for now
    let transport = libp2p::development_transport(peer.key.clone())
        .await
        .unwrap();

    // Gossipsub configuration
    let gossip_config = libp2p::gossipsub::GossipsubConfig::default();
    let message_authenticity = MessageAuthenticity::Signed(peer.key.clone());

    // Network Behaviours
    // Gossip to select the leader and to broadcast the epoch
    let gossip_behaviour = Gossipsub::new(message_authenticity, gossip_config).unwrap();

    // Rendezvous server to discover the leader
    let rendezvous_server_behaviour =
        rendezvous::server::Behaviour::new(rendezvous::server::Config::default());

    // Rendezvous client to select the leader
    let rendezvous_client_behaviour = rendezvous::client::Behaviour::new(peer.key.clone());


    // Swarm
    let mut swarm = Swarm::with_threadpool_executor(
        transport,
        HubBehaviour {
            rendezvous_server: rendezvous_server_behaviour,
            gossipsub: gossip_behaviour,
            rendezvous_client: rendezvous_client_behaviour,
        },
        peer.address,
    );


    // Implementing gossipsub
    let topic = libp2p::gossipsub::IdentTopic::new("Epoch");
    swarm.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

    swarm
        .listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap())
        .unwrap();

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
    let mut orchestrator_event_tracker  = 
            loop { 
                select! {
                    line = stdin.select_next_some() => 
                        match line {
                            Ok(line) => {
                                let message = line.as_bytes().to_vec();
                                swarm.behaviour_mut().gossipsub.publish(topic.clone(), message).unwrap();
                            },
                            Err(e) => {
                                println!("Error reading from stdin: {:?}", e);
                            }
                        },
                    // Read a line from stdin
                     event = swarm.select_next_some() =>
                      match event {
                         SwarmEvent::Behaviour(HubEvent::RendezvousServer(
                            rendezvous::server::Event::PeerRegistered { peer, registration }
                        )) => {
                             println!("Peer {} registered: {:?}", peer, registration);
        
                        },
                         // Print the peer id when started listening on a new address
                        SwarmEvent::NewListenAddr { address, .. } => {
                                println!("Listening on {:?}", address);
                            },
                        
                        SwarmEvent::Behaviour(HubEvent::RendezvousServer(
                            rendezvous::server::Event::PeerUnregistered { peer, namespace }
                        )) => {
                             println!("Peer {} unregistered from namespace: {:?}", peer, namespace);
                        },
                        SwarmEvent::Behaviour(HubEvent::RendezvousServer(
                            rendezvous::server::Event::DiscoverServed { enquirer, registrations}
                        )) => {
                             println!("Served discover request from {}: {:?}", enquirer, registrations);
                         },
                         // Register a peer when connected with the swarm
                         SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                println!("Peer {} connected", peer_id);
                                // List the registered peers in the rendezvous point
                                swarm.behaviour_mut().gossipsub.all_peers().for_each(|peer| {
                                    println!("Peer: {:?}", peer);
                                });
                            },
                          // Swarm event for gossipsub
                         SwarmEvent::Behaviour(HubEvent::Gossipsub(GossipsubEvent::Subscribed { peer_id, topic })) => {
                             println!("Peer {} subscribed to topic {:?}", peer_id, topic);
                             swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                         },
                         _ => {println!("Unhandled event: {:?}", event)}
                     }
                }
              };
        }




#[derive(Debug)]
pub enum HubEvent {
    RendezvousServer(rendezvous::server::Event),
    Gossipsub(GossipsubEvent),
    RendezvousClient(rendezvous::client::Event),
}

impl From<rendezvous::server::Event> for HubEvent {
    fn from(event: rendezvous::server::Event) -> Self {
        HubEvent::RendezvousServer(event)
    }
}
impl From<GossipsubEvent> for HubEvent {
    fn from(event: GossipsubEvent) -> Self {
        HubEvent::Gossipsub(event)
    }
}
impl From<rendezvous::client::Event> for HubEvent {
    fn from(event: rendezvous::client::Event) -> Self {
        HubEvent::RendezvousClient(event)
    }
}

// Use the derive macro to create a NetworkBehaviour that automatically
// delegates to all fields that implement NetworkBehaviour.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "HubEvent", event_process = false)]
pub struct HubBehaviour {
    rendezvous_server: rendezvous::server::Behaviour,
    gossipsub: Gossipsub,
    rendezvous_client: rendezvous::client::Behaviour,
}
