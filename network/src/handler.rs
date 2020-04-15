use std::{task::Poll};
use std::pin::Pin;

use futures::{channel::mpsc, channel::oneshot, future, prelude::*};
use libp2p::{
    floodsub::{self, Topic},
    multiaddr::{self},
    PeerId,
    swarm::SwarmEvent,
};

use map_core::{block::Block, transaction::Transaction};

use crate::{behaviour::{Behaviour, BehaviourEvent}, config, executor::NetworkMessage, NetworkConfig};
use crate::error;

impl Unpin for Service {}

type Swarm = libp2p::swarm::Swarm<Behaviour>;

/// The configuration and state of the libp2p components
pub struct Service {
    /// The libp2p Swarm handler.
    pub swarm: Swarm,
    /// This node's PeerId.
    local_peer_id: PeerId,
    network_recv: mpsc::UnboundedReceiver<NetworkMessage>,
}

impl Service {
    pub fn new(cfg: NetworkConfig, network_recv: mpsc::UnboundedReceiver<NetworkMessage>) -> error::Result<Self> {
        // Load the private key from CLI disk or generate a new random PeerId
        let local_key = config::load_private_key(&cfg);
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {:?}", local_peer_id);

        // Set up a an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
        let transport = libp2p::build_development_transport(local_key.clone()).expect("build transport error");

        // Create a Floodsub topic
        let floodsub_topic = floodsub::Topic::new("map");

        // Create a Swarm to manage peers and events
        let mut swarm = {
            let mut behaviour = Behaviour::new(&local_key)?;
            behaviour.floodsub.subscribe(floodsub_topic.clone());
            Swarm::new(transport, behaviour, local_peer_id.clone())
        };

        // Listen on listen_address
        match Swarm::listen_on(&mut swarm, cfg.listen_address.clone()) {
            Ok(_) => {
                let mut log_address = cfg.listen_address;
                log_address.push(multiaddr::Protocol::P2p(local_peer_id.clone().into()));
                info!("Listening established address {:?} ", format!("{}", log_address));
            }
            Err(err) => warn!(
                "Cannot listen on: {} because: {:?}", cfg.listen_address, err
            ),
        };

        // attempt to connect to cli p2p nodes
        for addr in cfg.dial_addrs {
            println!("dial {}", addr);
            match Swarm::dial_addr(&mut swarm, addr.clone()) {
                Ok(()) => info!("Dialing p2p peer address => {:?} ", addr),
                Err(err) => debug!(
                    "Could not connect to peer address {}", format!("{:?} Error {:?}", addr, err)),
            };
        }

        Ok(Service {
            local_peer_id: local_peer_id,
            swarm,
            network_recv,
        })
    }
}

//futures::future::Future
impl futures::future::Future for Service {
    //Future<Item=Foo, Error=Bar>
    //Future<Output=Result<Foo, Bar>>
    type Output = Result<Libp2pEvent, crate::error::Error>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        let this = &mut *self;
        println!("futures::future::Future poll");

        loop {
            // poll the network channel
            println!("futures::future::Future poll poll");
            let msg = match this.network_recv.poll_next_unpin(cx) {
                Poll::Ready(Some(msg)) => msg,
                Poll::Ready(None) => {
                    println!("spawn_service msg poll_next_unpin");
                    break;
                }
                Poll::Pending => {
                    println!("spawn_service msg poll_next_unpin Pending");
                    break;
                }
            };

            println!("spawn_service msg network_recv");
            match msg {
                NetworkMessage::Publish { topics, message } => {
                    debug!("Sending pubsub message topics {:?}", format!("{:?}", topics));
                    for topic in topics {
                        this.swarm.floodsub.publish(topic, message.clone());
                    }
                }
            }
            println!("spawn_service network_recv");
        }

        loop {
            // Process the next action coming from the network.
            let next_event = this.swarm.next_event();
            futures::pin_mut!(next_event);
            let poll_value = next_event.poll_unpin(cx);

            match poll_value {
                //Behaviour events
                Poll::Ready(SwarmEvent::Behaviour(event)) => match event {
                    BehaviourEvent::PubsubMessage {
                        source,
                        topics,
                        message,
                    } => {
                        //debug!(self.log, "Gossipsub message received"; "Message" => format!("{:?}", topics[0]));
                        return Poll::Ready(Ok(Libp2pEvent::PubsubMessage {
                            source,
                            topics,
                            message,
                        }));
                    }
                    BehaviourEvent::AnnounceBlock(peer_id, event) => {
                        //debug!(self.log,"Received RPC message from: {:?}", peer_id);
                        return Poll::Ready(Ok(Libp2pEvent::AnnounceBlock(peer_id, event)));
                    }
                    BehaviourEvent::PeerDialed(peer_id) => {
                        return Poll::Ready(Ok(Libp2pEvent::PeerDialed(peer_id)));
                    }
                    BehaviourEvent::PeerDisconnected(peer_id) => {
                        return Poll::Ready(Ok(Libp2pEvent::PeerDisconnected(peer_id)));
                    }
                },
                Poll::Pending => break,
                _ => break,
            }
        }
        println!("spawn_service msg network_recv over");

        for addr in Swarm::listeners(&this.swarm) {
            println!("Listening on {:?}", addr);
        }

        Poll::Pending
    }
}

/// Events that can be obtained from polling the Libp2p Service.
#[derive(Debug)]
pub enum Libp2pEvent {
    /// An RPC response request has been received on the swarm.
    AnnounceBlock(PeerId, Block),
    /// Initiated the connection to a new peer.
    PeerDialed(PeerId),
    /// A peer has disconnected.
    PeerDisconnected(PeerId),
    /// Received pubsub message.
    PubsubMessage {
        source: PeerId,
        topics: Vec<Topic>,
        message: Vec<u8>,
    },
}