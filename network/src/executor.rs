use std::{error::Error, task::{Context, Poll}};
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use async_std::{io, task};
use futures::{channel::mpsc, channel::oneshot, future, prelude::*};
use libp2p::{
    floodsub::{Floodsub, Topic},
};
use parking_lot::Mutex;

use chain::blockchain::BlockChain;

use crate::{
    handler::{Libp2pEvent, Service}, NetworkConfig};
use crate::error;
use core::ops::{DerefMut};

pub struct NetworkExecutor {
    service: Arc<Mutex<Service>>,
    exit_signal: oneshot::Sender<()>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
}

impl NetworkExecutor {
    pub fn new(cfg: NetworkConfig, block_chain: Arc<RwLock<BlockChain>>) -> error::Result<Self> {
        // build the network channel
        let ( network_send,  network_recv) = mpsc::unbounded::<NetworkMessage>();
        // launch libp2p Network
        let service = Arc::new(Mutex::new(Service::new(cfg)?));
        let libp2p_exit = spawn_service(
            service.clone(),
            network_recv,
            block_chain,
        )?;

        let network_service = NetworkExecutor {
            service,
            exit_signal: libp2p_exit,
            network_send,
        };

        Ok(network_service)
    }

    pub fn gossip(&mut self, topic: String, data: Vec<u8>) {
        self.network_send
            .send(NetworkMessage::Publish {
                topics: vec![Topic::new(topic)],
                message: data,
            });
    }
}

fn spawn_service(
    libp2p_service: Arc<Mutex<Service>>,
    mut network_recv: mpsc::UnboundedReceiver<NetworkMessage>,
    block_chain: Arc<RwLock<BlockChain>>,
) -> error::Result<futures::channel::oneshot::Sender<()>> {
    let (sender, receiver) = futures::channel::oneshot::channel();

    // spawn on the current executor
    task::spawn(future::poll_fn(move |cx: &mut Context| {
        loop {
            // poll the network channel
            let msg = match network_recv.poll_next_unpin(cx) {
                Poll::Ready(Some(msg)) => msg,
                Poll::Ready(None) => return Poll::Ready(()),
                Poll::Pending => break,
            };
            match msg {
                NetworkMessage::Publish { topics, message } => {
                    debug!("Sending pubsub message topics {:?}", format!("{:?}", topics));
                    for topic in topics {
                        libp2p_service.lock().swarm.floodsub.publish(topic, message.clone());
                    }
                }
            }
        }

        loop {
            // Process the next action coming from the network.

            match Future::poll(Pin::new(&mut libp2p_service.lock().deref_mut()), cx) {
                Poll::Pending => break,
                Poll::Ready(Ok(Libp2pEvent::PubsubMessage {
                                                      source,
                                                      topics, message, })) => {
                    debug!("Gossip message received: {:?}", message);
                    //-----------------------------------------  block chain
                }
                Poll::Ready(Ok(Libp2pEvent::PeerDialed(peer_id))) => {
                    debug!("Peer Dialed: {:?}", peer_id);
                }
                Poll::Ready(Ok(Libp2pEvent::PeerDisconnected(peer_id))) => {
                    debug!("Peer Disconnected: {:?}", peer_id);
                }
                Poll::Ready(Ok(e)) => {
                    info!("Network ok : {:?}", e);
                }
                Poll::Ready(Err(e)) => {
                    info!("Network error : {:?}", e);
                }
            };
        }
        Poll::Pending
    }));


    Ok(sender)
}

//Future<Item=Foo, Error=Bar>
//Future<Output=Result<Foo, Bar>>
/// Types of messages that the network Network can receive.
#[derive(Debug)]
pub enum NetworkMessage {
    /// Publish a message to pubsub mechanism.
    Publish {
        topics: Vec<Topic>,
        message: Vec<u8>,
    },
}