use std::{error::Error, task::{Context, Poll}, thread};
use std::sync::{Arc, RwLock};

use futures::{future, Future, Stream};
use futures::prelude::*;
use libp2p::{
    gossipsub::{Topic, TopicHash,MessageId},
};
use parking_lot::Mutex;
use slog::{debug, Drain, info, o, warn};
use tokio::runtime::Runtime;
use tokio::runtime::TaskExecutor;
use tokio::sync::{mpsc, oneshot};

use chain::blockchain::BlockChain;
use map_core::block::Block;
use map_core::types::Hash;

use crate::{
    {behaviour::{Behaviour, BehaviourEvent, PubsubMessage}
    },
    GossipTopic,
    handler::{Libp2pEvent, Service},
    NetworkConfig,
    PeerId,
};
use crate::error;

pub struct NetworkExecutor {
    service: Arc<Mutex<Service>>,
    pub exit_signal: oneshot::Sender<i32>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
    log: slog::Logger,
}

impl NetworkExecutor {
    pub fn new(cfg: NetworkConfig, block_chain: Arc<RwLock<BlockChain>>) -> error::Result<Self> {
        // build the network channel
        let (network_send, network_recv) = mpsc::unbounded_channel::<NetworkMessage>();
        // launch libp2p Network

        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, o!());

        let service = Arc::new(Mutex::new(Service::new(cfg, log.clone())?));

        let exit_signal = start_service(
            service.clone(),
            network_recv,
            block_chain,
            log.clone(),
        )?;

        let network_service = NetworkExecutor {
            service,
            exit_signal,
            network_send,
            log,
        };

        Ok(network_service)
    }

    pub fn gossip(&mut self, data: Block) {
        // create the network topic to send on
        let topic = GossipTopic::MapBlock;
        let message = PubsubMessage::Block(bincode::serialize(&data).unwrap());
        self.network_send
            .try_send(NetworkMessage::Publish {
                topics: vec![topic.into()],
                message,
            })
            .unwrap_or_else(|_| warn!(self.log, "Could not send gossip message."));
    }
}

fn start_service(
    libp2p_service: Arc<Mutex<Service>>,
    network_recv: mpsc::UnboundedReceiver<NetworkMessage>,
    block_chain: Arc<RwLock<BlockChain>>,
    log: slog::Logger,
) -> error::Result<tokio::sync::oneshot::Sender<i32>> {
    let (sender, exit_rx) = tokio::sync::oneshot::channel::<i32>();

    thread::spawn(move || {
        // spawn on the current executor
        tokio::run(
            network_service(
                libp2p_service,
                network_recv,
                block_chain,
                log.clone(),
            )
                // allow for manual termination
                .select(exit_rx.then(|_| Ok(())))
                .then(move |_| {
                    info!(log, "Stop p2p network");
                    Ok(())
                }),
        );
    });

    Ok(sender)
}

fn network_service(
    libp2p_service: Arc<Mutex<Service>>,
    mut network_recv: mpsc::UnboundedReceiver<NetworkMessage>,
    block_chain: Arc<RwLock<BlockChain>>,
    log: slog::Logger,
) -> impl futures::Future<Item=(), Error=error::Error> {
    futures::future::poll_fn(move || -> Result<_, error::Error> {
        loop {
            // poll the network channel
            match network_recv.poll() {
                Ok(Async::Ready(Some(message))) => match message {
                    NetworkMessage::Publish { topics, message } => {
                        debug!(log, "Sending pubsub message"; "topics" => format!("{:?}",topics));
                        libp2p_service.lock().swarm.publish(&topics, message.clone());
                    }
                    NetworkMessage::HandShake(_) => {}
                },
                Ok(Async::NotReady) => break,
                Ok(Async::Ready(None)) => {
                    return Err(error::Error::from("Network channel closed"));
                }
                Err(_) => {
                    return Err(error::Error::from("Network channel error"));
                }
            }
        }

        loop {
            // poll the swarm
            match libp2p_service.lock().poll() {
                Ok(Async::Ready(Some(event))) => match event {
                    Libp2pEvent::PubsubMessage {
                        id,
                        source,
                        message,
                        ..
                    } => {
                        handle_gossip(id, source, message, block_chain.clone(), libp2p_service.clone(),log.clone());
                    }
                    Libp2pEvent::PeerDialed(peer_id) => {
                        debug!(log, "Peer Dialed: {:?}", peer_id);
                    }
                    Libp2pEvent::PeerDisconnected(peer_id) => {
                        debug!(log, "Peer Disconnected: {:?}", peer_id);
                    }
                },
                Ok(Async::Ready(None)) => unreachable!("Stream never ends"),
                Ok(Async::NotReady) => break,
                Err(_) => break,
            }
        }

        Ok(Async::NotReady)
    })
}

/// Handle RPC messages
fn handle_gossip(id: MessageId, peer_id: PeerId, gossip_message: PubsubMessage, block_chain: Arc<RwLock<BlockChain>>,
                 libp2p_service: Arc<Mutex<Service>>, log :slog::Logger) {
    match gossip_message {
        PubsubMessage::Block(message) => match bincode::deserialize(&message[..]) {
            Ok(block) => {
                debug!(log, "Gossip message received: {:?}", block);
                match block_chain.write().expect("").insert_block(block) {
                    Ok(_) => {
                        libp2p_service.lock().swarm.propagate_message(&peer_id, id);
                    },
                    Err(e) => println!("network insert_block,Error: {:?}", e),
                }
            }
            Err(e) => {
                debug!(log, "Invalid gossiped block"; "peer_id" => format!("{}", peer_id), "Error" => format!("{:?}", e));
            }
        },
        PubsubMessage::Unknown(message) => {
            // Received a message from an unknown topic. Ignore for now
            debug!(log, "Unknown Gossip Message"; "peer_id" => format!("{}", peer_id), "Message" => format!("{:?}", message));
        }
    }
}

//Future<Item=Foo, Error=Bar>
//Future<Output=Result<Foo, Bar>>
/// Types of messages that the network Network can receive.
#[derive(Debug)]
pub enum NetworkMessage {
    HandShake(HandShakeMsg),
    /// Publish a message to pubsub mechanism.
    Publish {
        topics: Vec<Topic>,
        message: PubsubMessage,
    },
}

#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct HandShakeMsg {
    pub networkID: u16,
    pub genesisHash: Hash,
    hash: Hash,
    height: u128,
}

const STATUS_MSG: u32 = 1;
const Block_MSG: u32 = 2;
const Tx_MSG: u32 = 3;