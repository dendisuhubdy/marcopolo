use std::{thread};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use futures::{Future, Stream};
use futures::prelude::*;
use libp2p::{
    gossipsub::{MessageId, Topic},
    Swarm,
    multiaddr::Multiaddr,

};
use parking_lot::Mutex;
use slog::{debug, Drain, info, o, warn,trace};
use tokio::runtime::{Builder as RuntimeBuilder, Runtime, TaskExecutor};
use tokio::sync::{mpsc, oneshot};
use tokio::timer::Delay;

use chain::blockchain::BlockChain;
use map_core::block::Block;

use crate::{
    {behaviour::{PubsubMessage}
    },
    GossipTopic,
    NetworkConfig,
    PeerId,
    service::{Libp2pEvent, Service},
};
use crate::error;
use crate::handler::{HandlerMessage, MessageHandler};
use crate::p2p::{P2PEvent,P2PRequest};

/// The time in seconds that a peer will be banned and prevented from reconnecting.
const BAN_PEER_TIMEOUT: u64 = 30;

pub struct NetworkExecutor {
    service: Arc<Mutex<Service>>,
    pub exit_signal: oneshot::Sender<i32>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
    log: slog::Logger,
    pub runtime: Runtime,
}

impl NetworkExecutor {
    pub fn new(cfg: NetworkConfig, block_chain: Arc<RwLock<BlockChain>>, log_level: String) -> error::Result<Self> {
        // build the network channel
        let (network_send, network_recv) = mpsc::unbounded_channel::<NetworkMessage>();
        // launch libp2p Network

        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::CompactFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        let log = slog::Logger::root(drain, o!());

        let runtime = RuntimeBuilder::new()
            .core_threads(1)
            .build()
            .map_err(|e| format!("Failed to start runtime: {:?}", e))?;

        let executor: TaskExecutor = runtime.executor();

        let message_handler_send = MessageHandler::spawn(
            block_chain.clone(),
            network_send.clone(),
            &executor,
            log.clone(),
        )?;

        let service = Arc::new(Mutex::new(Service::new(cfg, log.clone())?));

        // A delay used to initialise code after the network has started
        // This is currently used to obtain the listening addresses from the libp2p service.
        let initial_delay = Delay::new(Instant::now() + Duration::from_secs(1));

        let exit_signal = start_service(
            service.clone(),
            network_recv,
            message_handler_send,
            block_chain,
            initial_delay,
            log.clone(),
        )?;

        let network_service = NetworkExecutor {
            service,
            exit_signal,
            network_send,
            log,
            runtime,
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
    message_handler_send: mpsc::UnboundedSender<HandlerMessage>,
    block_chain: Arc<RwLock<BlockChain>>,
    mut initial_delay: Delay,
    log: slog::Logger,
) -> error::Result<tokio::sync::oneshot::Sender<i32>> {
    let (sender, exit_rx) = tokio::sync::oneshot::channel::<i32>();

    thread::spawn(move || {
        // spawn on the current executor
        tokio::run(
            network_service(
                libp2p_service,
                network_recv,
                message_handler_send,
                block_chain,
                initial_delay,
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
    mut message_handler_send: mpsc::UnboundedSender<HandlerMessage>,
    block_chain: Arc<RwLock<BlockChain>>,
    mut initial_delay: Delay,
    log: slog::Logger,
) -> impl futures::Future<Item=(), Error=()> {
    futures::future::poll_fn(move || -> Result<_, ()> {
        if !initial_delay.is_elapsed() {
            if let Ok(Async::Ready(_)) = initial_delay.poll() {
                let multi_addrs : Vec<Multiaddr> = Swarm::listeners(&libp2p_service.lock().swarm).cloned().collect();
                println!("multi_addrs {:?}", multi_addrs)
            }
        }

        loop {
            // poll the network channel
            match network_recv.poll() {
                Ok(Async::Ready(Some(message))) => match message {
                    NetworkMessage::Publish { topics, message } => {
                        debug!(log, "Sending pubsub message"; "topics" => format!("{:?}",topics));
                        libp2p_service.lock().swarm.publish(&topics, message.clone());
                    }
                    NetworkMessage::P2P(peer_id, rpc_event) => {
                        trace!(log, "Sending RPC"; "rpc" => format!("{}", rpc_event));
                        libp2p_service.lock().swarm.send_rpc(peer_id, rpc_event);
                    }
                    NetworkMessage::Propagate {
                        propagation_source,
                        message_id,
                    } => {
                        trace!(log, "Propagating gossipsub message";
                            "propagation_peer" => format!("{:?}", propagation_source),
                            "message_id" => message_id.to_string(),
                            );
                        libp2p_service.lock()
                            .swarm
                            .propagate_message(&propagation_source, message_id);
                    }
                    NetworkMessage::Disconnect { peer_id } => {
                        libp2p_service.lock().disconnect_and_ban_peer(
                            peer_id,
                            std::time::Duration::from_secs(BAN_PEER_TIMEOUT),
                        );
                    }
                },
                Ok(Async::NotReady) => break,
                Ok(Async::Ready(None)) => {
                    debug!(log, "Network channel closed");
                    return Err(());
                }
                Err(e) => {
                    debug!(log, "Network channel error"; "error" => format!("{}", e));
                    return Err(());
                }
            }
        }

        let mut peers_to_ban = Vec::new();
        loop {
            // poll the swarm
            match libp2p_service.lock().poll() {
                Ok(Async::Ready(Some(event))) => match event {
                    Libp2pEvent::RPC(peer_id, rpc_event) => {
                        // trace!(log, "Received RPC"; "rpc" => format!("{}", rpc_event));

                        // if we received a Goodbye message, drop and ban the peer
                        if let P2PEvent::Request(_, P2PRequest::Goodbye(_)) = rpc_event {
                            peers_to_ban.push(peer_id.clone());
                        };
                        message_handler_send
                            .try_send(HandlerMessage::RPC(peer_id, rpc_event))
                            .map_err(|_| { debug!(log, "Failed to send RPC to handler"); })?;
                    }
                    Libp2pEvent::PubsubMessage {
                        id,
                        source,
                        message,
                        ..
                    } => {
                        message_handler_send
                            .try_send(HandlerMessage::PubsubMessage(id, source, message))
                            .map_err(|_| { debug!(log, "Failed to send pubsub message to handler"); })?;
                    }
                    Libp2pEvent::PeerDialed(peer_id) => {
                        debug!(log, "Peer Dialed"; "peer_id" => format!("{:?}", peer_id));
                        message_handler_send
                            .try_send(HandlerMessage::PeerDialed(peer_id))
                            .map_err(|_| { debug!(log, "Failed to send peer dialed to handler"); })?;
                    }
                    Libp2pEvent::PeerDisconnected(peer_id) => {
                        debug!(log, "Peer Disconnected";  "peer_id" => format!("{:?}", peer_id));
                        message_handler_send
                            .try_send(HandlerMessage::PeerDisconnected(peer_id))
                            .map_err(|_| { debug!(log, "Failed to send peer disconnect to handler"); })?;
                    }
                },
                Ok(Async::Ready(None)) => unreachable!("Stream never ends"),
                Ok(Async::NotReady) => break,
                Err(_) => break,
            }
        }

        // ban and disconnect any peers that sent Goodbye requests
        while let Some(peer_id) = peers_to_ban.pop() {
            libp2p_service.lock().disconnect_and_ban_peer(
                peer_id.clone(),
                std::time::Duration::from_secs(BAN_PEER_TIMEOUT),
            );
        }

        Ok(Async::NotReady)
    })
}

//Future<Item=Foo, Error=Bar>
//Future<Output=Result<Foo, Bar>>
/// Types of messages that the network Network can receive.
#[derive(Debug)]
pub enum NetworkMessage {
    /// Send an RPC message to the libp2p service.
    P2P(PeerId, P2PEvent),
    /// Publish a message to gossipsub.
    Publish {
        topics: Vec<Topic>,
        message: PubsubMessage,
    },
    /// Propagate a received gossipsub message.
    Propagate {
        propagation_source: PeerId,
        message_id: MessageId,
    },
    /// Disconnect and bans a peer id.
    Disconnect { peer_id: PeerId },
}