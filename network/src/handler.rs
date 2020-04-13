use std::{error::Error, task::{Context, Poll}};
use std::sync::{Arc, RwLock};

use async_std::{io, task};
use bincode;
use futures::{channel::mpsc, future, prelude::*};
use libp2p::{
    floodsub::{self, Floodsub},
    identity,
    mdns::Mdns,
    multiaddr::{self, Multiaddr},
    PeerId,
    ping::{Ping, PingConfig},
};

use chain::blockchain::BlockChain;
use map_core::{block::Block, transaction::Transaction};
use std::pin::Pin;
use crate::{behaviour::MyBehaviour, config, NetworkConfig};
use parking_lot::Mutex;

enum NetworkMessage {
    PropagateTx(Transaction),
    AnnounceBlock(Block),
}

impl Unpin for Service {
}

type Swarm = libp2p::swarm::Swarm<MyBehaviour>;
/// The configuration and state of the libp2p components
pub struct Service {
    /// The libp2p Swarm handler.
    pub swarm: Arc<Mutex<Swarm>>,
    /// This node's PeerId.
    local_peer_id: PeerId,

    network_send: mpsc::UnboundedSender<NetworkMessage>,
}

impl Service {

    pub fn start_network(cfg: NetworkConfig, block_chain: Arc<RwLock<BlockChain>>) -> Result<Self,Box<dyn Error>> {
        // Load the private key from CLI disk or generate a new random PeerId
        let local_key = config::load_private_key(&cfg);
        let local_peer_id = PeerId::from(local_key.public());
        println!("Local peer id: {:?}", local_peer_id);

        // Set up a an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
        let transport = libp2p::build_development_transport(local_key)?;

        // Create a Floodsub topic
        let floodsub_topic = floodsub::Topic::new("map");

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
            Swarm::new(transport, behaviour, local_peer_id.clone())
        };

        // attempt to connect to cli p2p nodes
        for addr in cfg.dial_addrs {
            println!("dial {}",addr);
            match Swarm::dial_addr(&mut swarm, addr.clone()) {
                Ok(()) => debug!("Dialing p2p peer address => {:?} ",  addr),
                Err(err) => debug!(
                    "Could not connect to peer address {}", format!("{:?} Error {:?}", addr,err)),
            };
        }

        // Listen on listen_address
        match Swarm::listen_on(&mut swarm, cfg.listen_address.clone()) {
            Ok(_) => {
                let mut log_address = cfg.listen_address;
                log_address.push(multiaddr::Protocol::P2p(local_peer_id.clone().into()));
                info!("Listening established address {:?} ",format!("{}", log_address));
            }
            Err(err) => warn!(
                "Cannot listen on: {} because: {:?}", cfg.listen_address, err
            ),
        };

        let (tx, rx) = mpsc::unbounded::<NetworkMessage>();
        let mut network_recv = rx;

        let swarm_service = Arc::new(Mutex::new(swarm));

        let swarm_close = swarm_service.clone();

        // Kick it off
        let mut listening = false;
        task::spawn(future::poll_fn(move |cx: &mut Context| {
            loop {
                match network_recv.poll_next_unpin(cx) {
                    Poll::Ready(Some(x)) => {
                        //Simulate pending transactions data
                        let block = block_chain.write().expect("network get block chain").current_block();
                        info!("Forwarding block");
                        swarm_close.lock().floodsub.publish(
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
                match swarm_close.lock().poll_next_unpin(cx) {
                    Poll::Ready(Some(event)) => println!("ready {:?}", event),
                    Poll::Ready(None) => return Poll::Ready(()),
                    Poll::Pending => {
                        if !listening {
                            for addr in Swarm::listeners(&swarm_close.lock()) {
                                println!("Listening on {:?}", addr);
                                listening = true;
                            }
                        }
                        break
                    }
                }
            }
            Poll::Pending
        }));


        Ok(Service {
            local_peer_id: local_peer_id,
            swarm:swarm_service,
            network_send : tx,
        })
    }
}

impl Future for Service {
    type Output = Result<(), io::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        let this = &mut *self;
        Poll::Pending
    }
}