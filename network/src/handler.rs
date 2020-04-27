use std::io::{Error, ErrorKind};
use std::io::prelude::*;
use std::time::Duration;

use futures::prelude::*;
use futures::Stream;
use libp2p::{gossipsub::{GossipsubMessage, Topic, TopicHash,MessageId}, multiaddr::Protocol, PeerId, Swarm};
use libp2p::core::{
    muxing::StreamMuxerBox,
    nodes::Substream,
    transport::boxed::Boxed,
};
use slog::{debug, info, warn,trace};

use map_core::{block::Block, transaction::Transaction};

use crate::{behaviour::{Behaviour, BehaviourEvent, PubsubMessage}, config, executor::NetworkMessage, GossipTopic, NetworkConfig, transport};
use crate::error;

type Libp2pStream = Boxed<(PeerId, StreamMuxerBox), Error>;
type Libp2pBehaviour = Behaviour<Substream<StreamMuxerBox>>;

/// The configuration and state of the libp2p components
pub struct Service {
    /// The libp2p Swarm handler.
    pub swarm: Swarm<Libp2pStream, Libp2pBehaviour>,
    /// This node's PeerId.
    local_peer_id: PeerId,
}

impl Service {
    pub fn new(cfg: NetworkConfig, log: slog::Logger) -> error::Result<Self> {
        // Load the private key from CLI disk or generate a new random PeerId
        let local_key = config::load_private_key(&cfg, log.clone());
        let local_peer_id = PeerId::from(local_key.public());
        info!(log, "Local peer id: {:?}", local_peer_id);

        // Create a Swarm to manage peers and events
        let mut swarm = {
            // Set up a an encrypted DNS-enabled TCP Transport over the Mplex and Yamux protocols
            let transport = transport::build_transport(local_key.clone());
            // network behaviour
            let behaviour = Behaviour::new(&local_key, &cfg, &log)?;
            Swarm::new(transport, behaviour, local_peer_id.clone())
        };


        // Listen on listen_address
        match Swarm::listen_on(&mut swarm, cfg.listen_address.clone()) {
            Ok(_) => {
                let mut log_address = cfg.listen_address;
                log_address.push(Protocol::P2p(local_peer_id.clone().into()));
                info!(log, "Listening established"; "address" => format!("{}", log_address));
            }
            Err(err) =>
                warn!(log, "Cannot listen on: {} because: {:?}", cfg.listen_address, err),
        };

        // attempt to connect to cli p2p nodes
        for addr in cfg.dial_addrs {
            println!("dial {}", addr);
            match Swarm::dial_addr(&mut swarm, addr.clone()) {
                Ok(()) => debug!(log, "Dialing p2p peer"; "address" => format!("{}", addr)),
                Err(err) =>
                    debug!(log,
                    "Could not connect to peer"; "address" => format!("{}", addr), "Error" => format!("{:?}", err)),
            };
        }

        // subscribe to default gossipsub topics
        let topics = vec![
            GossipTopic::MapBlock,
        ];

        let mut subscribed_topics: Vec<String> = vec![];
        for topic in topics {
            let raw_topic: Topic = topic.into();
            let topic_string = raw_topic.no_hash();
            if swarm.subscribe(raw_topic.clone()) {
                subscribed_topics.push(topic_string.as_str().into());
            } else {
                warn!(log, "Could not subscribe to topic"; "topic" => format!("{}",topic_string));
            }
        }
        info!(log, "Subscribed to topics"; "topics" => format!("{:?}", subscribed_topics));

        if let Some(a) = Swarm::listeners(&swarm).next() {
            println!("Listening on {:?}", a);
        }

        Ok(Service {
            local_peer_id: local_peer_id,
            swarm,
        })
    }
}

impl Stream for Service {
    type Item = Libp2pEvent;
    type Error = crate::error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.swarm.poll() {
                //Behaviour events
                Ok(Async::Ready(Some(event))) => match event {
                    BehaviourEvent::GossipMessage {
                        id,
                        source,
                        topics,
                        message,
                    } => {
                        return Ok(Async::Ready(Some(Libp2pEvent::PubsubMessage {
                            id,
                            source,
                            topics,
                            message,
                        })));
                    }
                    BehaviourEvent::PeerDialed(peer_id) => {
                        return Ok(Async::Ready(Some(Libp2pEvent::PeerDialed(peer_id))));
                    }
                    BehaviourEvent::PeerDisconnected(peer_id) => {
                        return Ok(Async::Ready(Some(Libp2pEvent::PeerDisconnected(peer_id))));
                    }
                },
                Ok(Async::Ready(None)) => unreachable!("Swarm stream shouldn't end"),
                Ok(Async::NotReady) => {
                    break;
                }
                _ => break,
            }
        }
        Ok(Async::NotReady)
    }
}

/// Events that can be obtained from polling the Libp2p Service.
#[derive(Debug)]
pub enum Libp2pEvent {
    /// Initiated the connection to a new peer.
    PeerDialed(PeerId),
    /// A peer has disconnected.
    PeerDisconnected(PeerId),
    /// Received pubsub message.
    PubsubMessage {
        id: MessageId,
        source: PeerId,
        topics: Vec<TopicHash>,
        message: PubsubMessage,
    },
}