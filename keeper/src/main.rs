use libp2p::Swarm;
use libp2p::gossipsub::{ MessageAuthenticity};
use libp2p::core::{identity::{Keypair}, transport::{Transport, MemoryTransport}};
use libp2p::noise::Keypair as NoiseKeypair;

fn main() {
   let localkey = Keypair::generate_ed25519();
   let local_peer_id = libp2p::PeerId::from(localkey.public());
   let noise_keypair = NoiseKeypair::<libp2p::noise::X25519Spec>::new()
     .into_authentic(&localkey)
     .unwrap();

   let transport = MemoryTransport::default()
     .upgrade(libp2p::core::upgrade::Version::V1).authenticate(
         libp2p::noise::NoiseConfig::xx(noise_keypair).into_authenticated()
     )
     .multiplex(libp2p::mplex::MplexConfig::new())
     .boxed()
     ;

   let topic = libp2p::gossipsub::IdentTopic::new("Epoch");
   let message_authenticity = MessageAuthenticity::Signed(localkey);
   let gossipsub_config = libp2p::gossipsub::GossipsubConfig::default();

   let mut gossipsub: libp2p::gossipsub::Gossipsub = 
     libp2p::gossipsub::Gossipsub::new(message_authenticity, gossipsub_config).unwrap();

   gossipsub.subscribe(&topic).unwrap();

  let mut swarm=  libp2p::swarm::Swarm::new(
      transport,
      gossipsub,
      local_peer_id
    );
  
  let memory: libp2p::core::Multiaddr = libp2p::multiaddr::Protocol::Memory(10).into();
  let addr = swarm.listen_on(memory).unwrap();
  println!("Listening on {:?}",addr);
  
  
}
