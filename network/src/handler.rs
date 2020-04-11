use async_std::{io, task};
use futures::{future, prelude::*,channel::mpsc};
use libp2p::{
    Multiaddr,
    PeerId,
    Swarm,
    identity,
    floodsub::{self, Floodsub},
    mdns::{Mdns},
    ping::{Ping, PingConfig},
};
use std::{error::Error, task::{Context, Poll}};
use crate::{behaviour::MyBehaviour,NetworkConfig, config};
use map_core::block::{Block};
use std::sync::{Arc, RwLock};
use chain::blockchain::BlockChain;
use bincode;

pub fn start_network(cfg: NetworkConfig, block_chain: Arc<RwLock<BlockChain>>) -> Result<(), Box<dyn Error>> {

    // Create a random PeerId
    let local_key = config::load_private_key(&cfg);
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up a an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::build_development_transport(local_key)?;

    // Create a Floodsub topic
    let floodsub_topic = floodsub::Topic::new("chat");

    // Create a Swarm to manage peers and events
    let mut swarm = {
        let mdns = Mdns::new()?;
        let mut behaviour = MyBehaviour {
            floodsub: Floodsub::new(local_peer_id.clone()),
            mdns,
            ping: Ping::new(PingConfig::new()),
            ignored_member: false,
        };

        behaviour.floodsub.subscribe(floodsub_topic.clone());
        Swarm::new(transport, behaviour, local_peer_id)
    };

    // Reach out to another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        let addr: Multiaddr = to_dial.parse()?;
        Swarm::dial_addr(&mut swarm, addr)?;
        println!("Dialed {:?}", to_dial)
    }

    // Listen on all interfaces and whatever port the OS assigns
    Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse()?)?;

    let (tx, rx) = mpsc::unbounded::<Block>();

    let mut network_recv = rx;

    // Kick it off
    let mut listening = false;
    task::block_on(future::poll_fn(move |cx: &mut Context| {
        loop {
            match network_recv.poll_next_unpin(cx) {
                Poll::Ready(Some(x)) => {
                    //Simulate pending transactions data
                    let block = block_chain.write().expect("network get block chain").current_block();
                    info!("Forwarding block");
                    swarm.floodsub.publish(
                        floodsub_topic.clone(),
                        bincode::serialize(&block).expect("Failed to serialize message."),
                    );
                }
                Poll::Ready(None) => panic!("Interval stream closed"),
                Poll::Pending => {
                    info!("send block Pending");
                    break
                }
            }
        }

        loop {
            match swarm.poll_next_unpin(cx) {
                Poll::Ready(Some(event)) => println!("ready {:?}", event),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => {
                    if !listening {
                        for addr in Swarm::listeners(&swarm) {
                            println!("Listening on {:?}", addr);
                            listening = true;
                        }
                    }
                    break
                }
            }
        }
        Poll::Pending
    }))
}
