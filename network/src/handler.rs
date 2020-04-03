use core::fmt::Debug;
use core::ops::DerefMut;
use std::time::{Duration, Instant};

use futures_util::{stream::Stream, io::{AsyncRead, AsyncWrite}};
use libp2p::{identity, PeerId};
use libp2p::swarm::{NetworkBehaviourAction, NetworkBehaviourEventProcess, PollParameters};
use libp2p::kad::Kademlia;
use libp2p::mdns::{Mdns, MdnsEvent};
use std::marker::PhantomData;
use libp2p::{
    Multiaddr,
    Swarm,
    NetworkBehaviour,
    floodsub::{self, Floodsub, FloodsubEvent},
};

pub fn start_network(port: &str) {

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up a an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::build_development_transport(local_key)?;

    // Create a Floodsub topic
    let floodsub_topic = floodsub::Topic::new("chat");

    let mut swarm = {
        let mut behaviour = DummyBehaviour{marker: PhantomData};

        libp2p::Swarm::new(transport, behaviour, local_peer_id)
    };

    // Listen on all interfaces and whatever port the OS assigns
    let addr = libp2p::Swarm::listen_on(&mut swarm, format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap()).unwrap();
    println!("Listening on {:?}", addr);

    let mut interval = Interval::new_interval(Duration::new(5, 0));
    let mut listening = false;
    tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
        loop {
            match interval.poll().expect("Error while polling interval") {
                Async::Ready(Some(_)) => {
                    //sync.on_tick(swarm.deref_mut());
                }
                Async::Ready(None) => panic!("Interval closed"),
                Async::NotReady => break,
            };
        }

        loop {
            match swarm.poll().expect("Error while polling swarm") {
                Async::Ready(Some((peer_id, message))) => {
                    println!("Received: {:?} from {:?}", message, peer_id);
                    //sync.on_message(swarm.deref_mut(), &peer_id, message);
                }
                Async::Ready(None) | Async::NotReady => {
                    if !listening {
                        if let Some(a) = libp2p::Swarm::listeners(&swarm).next() {
                            println!("Listening on {:?}", a);
                            listening = true;
                        }
                    }
                    break;
                }
            }
        }

        Ok(Async::NotReady)
    }));
}