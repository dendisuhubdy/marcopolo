use super::block_processor::{BatchProcessResult};
use super::network_context::SyncNetworkContext;
use super::range_sync::{BatchId, RangeSync};
use crate::handler_processor::PeerSyncInfo;
use crate::manager::NetworkMessage;
use crate::p2p::RequestId;
use libp2p::PeerId;
use futures::prelude::*;
use slog::{debug, error, info, trace, Logger};
use smallvec::SmallVec;
use std::boxed::Box;
use std::collections::HashSet;
use std::ops::Sub;
use tokio::sync::{mpsc, oneshot};
use chain::blockchain::BlockChain;
use std::sync::{Arc, RwLock};
use map_core::block::Block;

/// The number of slots ahead of us that is allowed before requesting a long-range (batch)  Sync
/// from a peer. If a peer is within this tolerance (forwards or backwards), it is treated as a
/// fully sync'd peer.
const SLOT_IMPORT_TOLERANCE: u64 = 20;
/// How many attempts we try to find a parent of a block before we give up trying .
const PARENT_FAIL_TOLERANCE: u64 = 3;
/// The maximum depth we will search for a parent block. In principle we should have sync'd any
/// canonical chain to its head once the peer connects. A chain should not appear where it's depth
/// is further back than the most recent head slot.
const PARENT_DEPTH_TOLERANCE: u64 = SLOT_IMPORT_TOLERANCE * 2;

#[derive(Debug)]
/// A message than can be sent to the sync manager thread.
pub enum SyncMessage {
    /// A useful peer has been discovered.
    AddPeer(PeerId, PeerSyncInfo),

    /// A `BlocksByRange` response has been received.
    BlocksByRangeResponse {
        peer_id: PeerId,
        request_id: RequestId,
        beacon_block: Option<Box<Block>>,
    },

    UnknownBlock(PeerId),

    /// A peer has disconnected.
    Disconnect(PeerId),

    /// An RPC Error has occurred on a request.
    RPCError(PeerId, RequestId),

    /// A batch has been processed by the block processor thread.
    BatchProcessed {
        batch_id: BatchId,
        downloaded_blocks: Vec<Block>,
        result: BatchProcessResult,
    },
}

/// Maintains a sequential list of parents to lookup and the lookup's current state.
struct ParentRequests {
    /// The blocks that have currently been downloaded.
    downloaded_blocks: Vec<Block>,

    /// The number of failed attempts to retrieve a parent block. If too many attempts occur, this
    /// lookup is failed and rejected.
    failed_attempts: usize,

    /// The peer who last submitted a block. If the chain ends or fails, this is the peer that is
    /// downvoted.
    last_submitted_peer: PeerId,

    /// The request ID of this lookup is in progress.
    pending: Option<RequestId>,
}

#[derive(PartialEq, Debug, Clone)]
/// The current state of the `ImportManager`.
enum ManagerState {
    /// The manager is performing a long-range (batch) sync. In this mode, parent lookups are
    /// disabled.
    Syncing,

    /// The manager is up to date with all known peers and is connected to at least one
    /// fully-syncing peer. In this state, parent lookups are enabled.
    Regular,

    /// No useful peers are connected. Long-range sync's cannot proceed and we have no useful
    /// peers to download parents for. More peers need to be connected before we can proceed.
    Stalled,
}

/// The primary object for handling and driving all the current syncing logic. It maintains the
/// current state of the syncing process, the number of useful peers, downloaded blocks and
/// controls the logic behind both the long-range (batch) sync and the on-going potential parent
/// look-up of blocks.
pub struct SyncManager {
    /// A weak reference to the underlying beacon chain.
    chain: Arc<RwLock<BlockChain>>,

    /// The current state of the import manager.
    state: ManagerState,

    /// A receiving channel sent by the message processor thread.
    input_channel: mpsc::UnboundedReceiver<SyncMessage>,

    /// A network context to contact the network service.
    network: SyncNetworkContext,

    /// The object handling long-range batch load-balanced syncing.
    range_sync: RangeSync,

    /// A collection of parent block lookups.
    parent_queue: SmallVec<[ParentRequests; 3]>,

    /// The collection of known, connected, fully-sync'd peers.
    full_peers: HashSet<PeerId>,

    /// The logger for the import manager.
    log: Logger,

    /// The sending part of input_channel
    sync_send: mpsc::UnboundedSender<SyncMessage>,
}

/// Spawns a new `SyncManager` thread which has a weak reference to underlying beacon
/// chain. This allows the chain to be
/// dropped during the syncing process which will gracefully end the `SyncManager`.
pub fn spawn(
    executor: &tokio::runtime::TaskExecutor,
    block_chain: Arc<RwLock<BlockChain>>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
    log: slog::Logger,
) -> (
    mpsc::UnboundedSender<SyncMessage>,
    oneshot::Sender<()>,
) {
    // generate the exit channel
    let (sync_exit, exit_rx) = tokio::sync::oneshot::channel();
    // generate the message channel
    let (sync_send, sync_recv) = mpsc::unbounded_channel::<SyncMessage>();

    // create an instance of the SyncManager
    let sync_manager = SyncManager {
        chain: block_chain.clone(),
        state: ManagerState::Stalled,
        input_channel: sync_recv,
        network: SyncNetworkContext::new(network_send, log.clone()),
        range_sync: RangeSync::new(block_chain, sync_send.clone(), log.clone()),
        parent_queue: SmallVec::new(),
        full_peers: HashSet::new(),
        log: log.clone(),
        sync_send: sync_send.clone(),
    };

    // spawn the sync manager thread
    debug!(log, "Sync Manager started");
    executor.spawn(
        sync_manager
            .select(exit_rx.then(|_| Ok(())))
            .then(move |_| {
                info!(log.clone(), "Sync Manager shutdown");
                Ok(())
            }),
    );
    (sync_send, sync_exit)
}

impl SyncManager {
    /* Input Handling Functions */

    /// A peer has connected which has blocks that are unknown to us.
    ///
    /// This function handles the logic associated with the connection of a new peer. If the peer
    /// is sufficiently ahead of our current head, a range-sync (batch) sync is started and
    /// batches of blocks are queued to download from the peer. Batched blocks begin at our latest
    /// finalized head.
    ///
    /// If the peer is within the `SLOT_IMPORT_TOLERANCE`, then it's head is sufficiently close to
    /// ours that we consider it fully sync'd with respect to our current chain.
    fn add_peer(&mut self, peer_id: PeerId, remote: PeerSyncInfo) {
        // ensure the beacon chain still exists
        let local = match PeerSyncInfo::from_chain(self.chain.clone()) {
            Some(local) => local,
            None => {
                return error!(
                    self.log,
                    "Failed to get peer sync info";
                    "msg" => "likely due to head lock contention"
                )
            }
        };

        // If a peer is within SLOT_IMPORT_TOLERANCE from our head slot, ignore a batch/range sync,
        // consider it a fully-sync'd peer.
        if remote.finalized_number.sub(local.finalized_number) < SLOT_IMPORT_TOLERANCE {
            trace!(self.log, "Ignoring full sync with peer";
            "peer" => format!("{:?}", peer_id),
            "peer_finalized_number" => remote.finalized_number,
            "local_finalized_number" => local.finalized_number,
            );
            self.add_full_peer(peer_id.clone());
        }

        // Add the peer to our RangeSync
        self.range_sync.add_peer(&mut self.network, peer_id, remote);
    }

    fn peer_disconnect(&mut self, peer_id: &PeerId) {
        self.range_sync.peer_disconnect(&mut self.network, peer_id);
        self.full_peers.remove(peer_id);
    }

    fn add_full_peer(&mut self, peer_id: PeerId) {
        debug!(
            self.log, "Fully synced peer added";
            "peer" => format!("{:?}", peer_id),
        );
        self.full_peers.insert(peer_id);
    }
}

impl Future for SyncManager {
    type Item = ();
    type Error = String;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        // process any inbound messages
        loop {
            match self.input_channel.poll() {
                Ok(Async::Ready(Some(message))) => match message {
                    SyncMessage::AddPeer(peer_id, info) => {
                        self.add_peer(peer_id, info);
                    }
                    SyncMessage::BlocksByRangeResponse {
                        peer_id,
                        request_id,
                        beacon_block,
                    } => {
                        self.range_sync.blocks_by_range_response(
                            &mut self.network,
                            peer_id,
                            request_id,
                            beacon_block.map(|b| *b),
                        );
                    }
                    SyncMessage::Disconnect(peer_id) => {
                        self.peer_disconnect(&peer_id);
                    }
                    SyncMessage::RPCError(peer_id, request_id) => {
                        println!("RPCError");
                    }
                    SyncMessage::UnknownBlock(peer_id) => {
                        println!("UnknownBlock {:?}", peer_id);
                    }
                    SyncMessage::BatchProcessed {
                        batch_id,
                        downloaded_blocks,
                        result,
                    } => {
                        self.range_sync.handle_block_process_result(
                            &mut self.network,
                            batch_id,
                            downloaded_blocks,
                            result,
                        );
                    }
                },
                Ok(Async::NotReady) => break,
                Ok(Async::Ready(None)) => {
                    return Err("Sync manager channel closed".into());
                }
                Err(e) => {
                    return Err(format!("Sync Manager channel error: {:?}", e));
                }
            }
        }

        Ok(Async::NotReady)
    }
}
