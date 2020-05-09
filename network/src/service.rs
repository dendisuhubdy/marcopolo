use std::collections::HashSet;
use std::io::{Error, ErrorKind};
use std::time::Duration;

use futures::prelude::*;
use futures::Stream;
use libp2p::{gossipsub::{MessageId, Topic, TopicHash}, multiaddr::Protocol, PeerId, Swarm};
use libp2p::core::{
    ConnectedPoint,
    multiaddr::Multiaddr,
    muxing::StreamMuxerBox,
    nodes::Substream,
    transport::boxed::Boxed,
};
use slog::{debug, error, info, warn};
use tokio::timer::DelayQueue;

use crate::{behaviour::{Behaviour, BehaviourEvent, PubsubMessage}, config, GossipTopic, NetworkConfig, transport};
use crate::error;
use crate::p2p::P2PEvent;

type Libp2pStream = Boxed<(PeerId, StreamMuxerBox), Error>;
type Libp2pBehaviour = Behaviour<Substream<StreamMuxerBox>>;

/// The time in milliseconds to wait before banning a peer. This allows for any Goodbye messages to be
/// flushed and protocols to be negotiated.
const BAN_PEER_WAIT_TIMEOUT: u64 = 200;

/// The configuration and state of the libp2p components
pub struct Service {
    /// The libp2p Swarm handler.
    pub swarm: Swarm<Libp2pStream, Libp2pBehaviour>,
    /// This node's PeerId.
    local_peer_id: PeerId,

    /// A current list of peers to ban after a given timeout.
    peers_to_ban: DelayQueue<PeerId>,

    /// A list of timeouts after which peers become unbanned.
    peer_ban_timeout: DelayQueue<PeerId>,
    pub peers: HashSet<PeerId>,
    pub log: slog::Logger,
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
            local_peer_id,
            swarm,
            peers_to_ban: DelayQueue::new(),
            peer_ban_timeout: DelayQueue::new(),
            peers: HashSet::new(),
            log,
        })
    }

    /// Adds a peer to be banned for a period of time, specified by a timeout.
    pub fn disconnect_and_ban_peer(&mut self, peer_id: PeerId, timeout: Duration) {
        error!(self.log, "Disconnecting and banning peer"; "peer_id" => format!("{:?}", peer_id), "timeout" => format!("{:?}", timeout));
        self.peers_to_ban.insert(
            peer_id.clone(),
            Duration::from_millis(BAN_PEER_WAIT_TIMEOUT),
        );
        self.peer_ban_timeout.insert(peer_id, timeout);
    }

    pub fn dial_peer(&mut self, addr: Multiaddr) -> bool {
        let result = match Swarm::dial_addr(&mut self.swarm, addr.clone()) {
            Ok(()) => {
                debug!(self.log, "Dialing p2p peer"; "address" => format!("{}", addr));
                true
            }
            Err(err) => {
                debug!(self.log,
                            "Could not connect to peer"; "address" => format!("{}", addr), "Error" => format!("{:?}", err));
                false
            }
        };
        result
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
                    BehaviourEvent::RPC(peer_id, event) => {
                        return Ok(Async::Ready(Some(Libp2pEvent::RPC(peer_id, event))));
                    }
                    BehaviourEvent::PeerDialed(peer_id) => {
                        self.peers.insert(peer_id.clone());
                        return Ok(Async::Ready(Some(Libp2pEvent::PeerDialed(peer_id))));
                    }
                    BehaviourEvent::PeerDisconnected(peer_id) => {
                        self.peers.remove(&peer_id);
                        return Ok(Async::Ready(Some(Libp2pEvent::PeerDisconnected(peer_id))));
                    }
                    BehaviourEvent::FindPeers { peer_id, addrs } => {
                        if !self.peers.contains(&peer_id) {
                            // attempt to connect to cli p2p nodes
                            for addr in addrs.into_vec() {
                                let addr_str = addr.to_string();
                                if addr_str.contains("127.0.0.1") || !addr_str.contains("ip4") {
                                    continue;
                                }
                                self.dial_peer(addr);
                            };
                        }
                    }
                },
                Ok(Async::Ready(None)) => unreachable!("Swarm stream shouldn't end"),
                Ok(Async::NotReady) => {
                    break;
                }
                _ => break,
            }
        }

        // check if peers need to be banned
        loop {
            match self.peers_to_ban.poll() {
                Ok(Async::Ready(Some(peer_id))) => {
                    let peer_id = peer_id.into_inner();
                    Swarm::ban_peer_id(&mut self.swarm, peer_id.clone());
                    let dummy_connected_point = ConnectedPoint::Dialer {
                        address: "/ip4/0.0.0.0"
                            .parse::<Multiaddr>()
                            .expect("valid multiaddr"),
                    };

                    self.swarm
                        .inject_disconnected(&peer_id, dummy_connected_point);
                }
                Ok(Async::NotReady) | Ok(Async::Ready(None)) => break,
                Err(e) => {
                    warn!(self.log, "Peer banning queue failed"; "error" => format!("{:?}", e));
                }
            }
        }

        // un-ban peer if it's timeout has expired
        loop {
            match self.peer_ban_timeout.poll() {
                Ok(Async::Ready(Some(peer_id))) => {
                    let peer_id = peer_id.into_inner();
                    debug!(self.log, "Peer has been unbanned"; "peer" => format!("{:?}", peer_id));
                    Swarm::unban_peer_id(&mut self.swarm, peer_id);
                }
                Ok(Async::NotReady) | Ok(Async::Ready(None)) => break,
                Err(e) => {
                    warn!(self.log, "Peer banning timeout queue failed"; "error" => format!("{:?}", e));
                }
            }
        }
        Ok(Async::NotReady)
    }
}

/// Events that can be obtained from polling the Libp2p Service.
#[derive(Debug)]
pub enum Libp2pEvent {
    /// An RPC response request has been received on the swarm.
    RPC(PeerId, P2PEvent),
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