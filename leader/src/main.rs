use futures::{prelude::*, select};
use libp2p::gossipsub::{Gossipsub, GossipsubEvent, MessageAuthenticity};
use libp2p::rendezvous;
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{identity, PeerId};
use std::io::{self, BufRead};
use std::sync::Arc;
// Import mutex
use futures::lock::Mutex;
// use futures 
use std::thread;
#[async_std::main]
async fn main() {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    // Create a Swarm to manage peers and events
    let transport = libp2p::development_transport(local_key.clone())
        .await
        .unwrap();
    let gossip_config = libp2p::gossipsub::GossipsubConfig::default();
    let message_authenticity = MessageAuthenticity::Signed(local_key.clone());

    // Network Behaviours
    let gossip_behaviour = Gossipsub::new(message_authenticity, gossip_config).unwrap();
    let rendezvous_server_behaviour =
        rendezvous::server::Behaviour::new(rendezvous::server::Config::default());
    let rendezvous_client_behaviour = rendezvous::client::Behaviour::new(local_key.clone());

    let mut swarm = Swarm::with_threadpool_executor(
        transport,
        HubBehaviour {
            rendezvous_server: rendezvous_server_behaviour,
            gossipsub: gossip_behaviour,
            rendezvous_client: rendezvous_client_behaviour,
        },
        local_peer_id,
    );

    let swarm = Arc::new(Mutex::new(swarm));
    let swarm_clone = swarm.clone();
    // Implementing gossipsub
    let topic = libp2p::gossipsub::IdentTopic::new("Epoch");

    swarm.lock().await.behaviour_mut().gossipsub.subscribe(&topic).unwrap();
    swarm.lock().await.behaviour_mut().gossipsub.publish(topic.clone(), "Hello".as_bytes()).unwrap();
    // let mut stdin = io::BufReader::new(io::stdin());
    // let mut line = String::new();
    // while let Ok(n) = stdin.read_line(&mut line) {
    //     if n == 0 {
    //         break;
    //     }
    //     // Send the message to all connected peers.
    //     swarm.lock().await
    //         .behaviour_mut()
    //         .gossipsub
    //         .publish(topic.clone(), line.as_bytes())
    //         .unwrap();
    //     // process the line
    //     line.clear();
    // }
    swarm.lock().await
        .listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap())
        .unwrap();

    // Use the disover method to find peers in every 20 seconds
    thread::spawn(move || 
        async move {
        let mut swarm_clone = swarm_clone.lock().await;
        loop {
        thread::sleep(std::time::Duration::from_secs(20));
        swarm_clone.behaviour_mut().rendezvous_client.discover(
            None,
            None,
            None,
            // Borrow the peer Id of the swarm mutably
          local_peer_id
        );
        
 } });
    
    loop {
        let mut swarm =  swarm.lock().await;
        select! {
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
                    },
                  // Swarm event for gossipsub
                 SwarmEvent::Behaviour(HubEvent::Gossipsub(GossipsubEvent::Subscribed { peer_id, topic })) => {
                     println!("Peer {} subscribed to topic {:?}", peer_id, topic);
                     swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                 },
                 _ => {println!("Unhandled event: {:?}", event)}
             }
        }
    }
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
