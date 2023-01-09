mod peer;
use peer::{Peer};
use futures::{prelude::*, select, lock::Mutex};
use libp2p::
{
    gossipsub::{Gossipsub, GossipsubEvent, MessageAuthenticity},
    rendezvous,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
};
use tokio;
// BufRead


use std::{thread, sync::Arc, io, io::BufRead};

#[async_std::main]
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
    let swarm = Swarm::with_threadpool_executor(
        transport,
        HubBehaviour {
            rendezvous_server: rendezvous_server_behaviour,
            gossipsub: gossip_behaviour,
            rendezvous_client: rendezvous_client_behaviour,
        },
        peer.address,
    );

    let swarm = Arc::new(Mutex::new(swarm));
    let swarm_clone = swarm.clone();

    // Implementing gossipsub
    let topic = libp2p::gossipsub::IdentTopic::new("Epoch");
    swarm.lock().await.behaviour_mut().gossipsub.subscribe(&topic).unwrap();

    swarm.lock().await
        .listen_on("/ip4/0.0.0.0/tcp/62649".parse().unwrap())
        .unwrap();

    // // Use the disover method to find peers in every 20 seconds
    tokio::spawn(async move {
        let mut swarm_clone = swarm_clone.lock().await;
        loop {
        thread::sleep(std::time::Duration::from_secs(20));
        println!("Discovering peers");
        let peers = swarm_clone.behaviour_mut().rendezvous_client.discover(
            None,
            None,
            None,
            peer.address.clone(),
        );
        println!("Peers discovered: {:?}", peers);
        } 
    }.await,
      );

     // Read full lines from stdin
     // Convert to a 
   let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

   std::thread::spawn(move || async {
        async move {
        loop {
            let mut swarm =  swarm.lock().await;
            select! {
                line = stdin.select_next_some() => {
                    if let Err(e) = swarm
                        .behaviour_mut().gossipsub
                        .publish(topic.clone(), line.expect("Stdin not to close").as_bytes()) {
                        println!("Publish error: {e:?}");
                 }
                },
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
    }.await;
});
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
