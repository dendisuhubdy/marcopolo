#![allow(clippy::type_complexity)]
#![allow(clippy::cognitive_complexity)]

use super::methods::{ErrorMessage, P2PErrorResponse, RequestId, ResponseTermination};
use super::protocol::{P2PError, P2PProtocol, P2PRequest};
use super::P2PEvent;
use crate::p2p::protocol::{InboundFramed, OutboundFramed};
use core::marker::PhantomData;
use fnv::FnvHashMap;
use futures::prelude::*;
use libp2p::core::upgrade::{InboundUpgrade, OutboundUpgrade, UpgradeError};
use libp2p::swarm::protocols_handler::{
    KeepAlive, ProtocolsHandler, ProtocolsHandlerEvent, ProtocolsHandlerUpgrErr, SubstreamProtocol,
};
use slog::{crit, debug, error, warn};
use smallvec::SmallVec;
use std::collections::hash_map::Entry;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::timer::{delay_queue, DelayQueue};

//TODO: Implement close() on the substream types to improve the poll code.
//TODO: Implement check_timeout() on the substream types

/// The time (in seconds) before a substream that is awaiting a response from the user times out.
pub const RESPONSE_TIMEOUT: u64 = 10;

/// The number of times to retry an outbound upgrade in the case of IO errors.
const IO_ERROR_RETRIES: u8 = 3;

/// Inbound requests are given a sequential `RequestId` to keep track of. All inbound streams are
/// identified by their substream ID which is identical to the P2P Id.
type InboundRequestId = RequestId;
/// Outbound requests are associated with an id that is given by the application that sent the
/// request.
type OutboundRequestId = RequestId;

/// Implementation of `ProtocolsHandler` for the P2P protocol.
pub struct P2PHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// The upgrade for inbound substreams.
    listen_protocol: SubstreamProtocol<P2PProtocol>,

    /// If something bad happened and we should shut down the handler with an error.
    pending_error: Vec<(RequestId, ProtocolsHandlerUpgrErr<P2PError>)>,

    /// Queue of events to produce in `poll()`.
    events_out: SmallVec<[P2PEvent; 4]>,

    /// Queue of outbound substreams to open.
    dial_queue: SmallVec<[P2PEvent; 4]>,

    /// Current number of concurrent outbound substreams being opened.
    dial_negotiated: u32,

    /// Current inbound substreams awaiting processing.
    inbound_substreams:
        FnvHashMap<InboundRequestId, (InboundSubstreamState<TSubstream>, Option<delay_queue::Key>)>,

    /// Inbound substream `DelayQueue` which keeps track of when an inbound substream will timeout.
    inbound_substreams_delay: DelayQueue<InboundRequestId>,

    /// Map of outbound substreams that need to be driven to completion. The `RequestId` is
    /// maintained by the application sending the request.
    outbound_substreams:
        FnvHashMap<OutboundRequestId, (OutboundSubstreamState<TSubstream>, delay_queue::Key)>,

    /// Inbound substream `DelayQueue` which keeps track of when an inbound substream will timeout.
    outbound_substreams_delay: DelayQueue<OutboundRequestId>,

    /// Map of outbound items that are queued as the stream processes them.
    queued_outbound_items: FnvHashMap<RequestId, Vec<P2PErrorResponse>>,

    /// Sequential ID for waiting substreams. For inbound substreams, this is also the inbound request ID.
    current_inbound_substream_id: RequestId,

    /// Maximum number of concurrent outbound substreams being opened. Value is never modified.
    max_dial_negotiated: u32,

    /// Value to return from `connection_keep_alive`.
    keep_alive: KeepAlive,

    /// After the given duration has elapsed, an inactive connection will shutdown.
    inactive_timeout: Duration,

    /// Try to negotiate the outbound upgrade a few times if there is an IO error before reporting the request as failed.
    /// This keeps track of the number of attempts.
    outbound_io_error_retries: u8,

    /// Logger for handling P2P streams
    log: slog::Logger,

    /// Marker to pin the generic stream.
    _phantom: PhantomData<TSubstream>,
}

/// State of an outbound substream. Either waiting for a response, or in the process of sending.
pub enum InboundSubstreamState<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// A response has been sent, pending writing and flush.
    ResponsePendingSend {
        /// The substream used to send the response
        substream: futures::sink::Send<InboundFramed<TSubstream>>,
        /// Whether a stream termination is requested. If true the stream will be closed after
        /// this send. Otherwise it will transition to an idle state until a stream termination is
        /// requested or a timeout is reached.
        closing: bool,
    },
    /// The response stream is idle and awaiting input from the application to send more chunked
    /// responses.
    ResponseIdle(InboundFramed<TSubstream>),
    /// The substream is attempting to shutdown.
    Closing(InboundFramed<TSubstream>),
    /// Temporary state during processing
    Poisoned,
}

pub enum OutboundSubstreamState<TSubstream> {
    /// A request has been sent, and we are awaiting a response. This future is driven in the
    /// handler because GOODBYE requests can be handled and responses dropped instantly.
    RequestPendingResponse {
        /// The framed negotiated substream.
        substream: OutboundFramed<TSubstream>,
        /// Keeps track of the actual request sent.
        request: P2PRequest,
    },
    /// Closing an outbound substream>
    Closing(OutboundFramed<TSubstream>),
    /// Temporary state during processing
    Poisoned,
}

impl<TSubstream> InboundSubstreamState<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    /// Moves the substream state to closing and informs the connected peer. The
    /// `queued_outbound_items` must be given as a parameter to add stream termination messages to
    /// the outbound queue.
    pub fn close(&mut self, outbound_queue: &mut Vec<P2PErrorResponse>) {
        // When terminating a stream, report the stream termination to the requesting user via
        // an P2P error
        let error = P2PErrorResponse::ServerError(ErrorMessage {
            error_message: b"Request timed out".to_vec(),
        });

        // The stream termination type is irrelevant, this will terminate the
        // stream
        let stream_termination =
            P2PErrorResponse::StreamTermination(ResponseTermination::BlocksByRange);

        match std::mem::replace(self, InboundSubstreamState::Poisoned) {
            InboundSubstreamState::ResponsePendingSend { substream, closing } => {
                if !closing {
                    outbound_queue.push(error);
                    outbound_queue.push(stream_termination);
                }
                // if the stream is closing after the send, allow it to finish

                *self = InboundSubstreamState::ResponsePendingSend { substream, closing }
            }
            InboundSubstreamState::ResponseIdle(mut substream) => {
                // check if the stream is already closed
                if let Ok(Async::Ready(None)) = substream.poll() {
                    *self = InboundSubstreamState::Closing(substream);
                } else {
                    *self = InboundSubstreamState::ResponsePendingSend {
                        substream: substream.send(error),
                        closing: true,
                    };
                }
            }
            InboundSubstreamState::Closing(substream) => {
                // let the stream close
                *self = InboundSubstreamState::Closing(substream);
            }
            InboundSubstreamState::Poisoned => {
                unreachable!("Coding error: Timeout poisoned substream")
            }
        };
    }
}

impl<TSubstream> P2PHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    pub fn new(
        listen_protocol: SubstreamProtocol<P2PProtocol>,
        inactive_timeout: Duration,
        log: &slog::Logger,
    ) -> Self {
        P2PHandler {
            listen_protocol,
            pending_error: Vec::new(),
            events_out: SmallVec::new(),
            dial_queue: SmallVec::new(),
            dial_negotiated: 0,
            queued_outbound_items: FnvHashMap::default(),
            inbound_substreams: FnvHashMap::default(),
            outbound_substreams: FnvHashMap::default(),
            inbound_substreams_delay: DelayQueue::new(),
            outbound_substreams_delay: DelayQueue::new(),
            current_inbound_substream_id: 1,
            max_dial_negotiated: 8,
            keep_alive: KeepAlive::Yes,
            inactive_timeout,
            outbound_io_error_retries: 0,
            log: log.clone(),
            _phantom: PhantomData,
        }
    }

    /// Returns the number of pending requests.
    pub fn pending_requests(&self) -> u32 {
        self.dial_negotiated + self.dial_queue.len() as u32
    }

    /// Returns a reference to the listen protocol configuration.
    ///
    /// > **Note**: If you modify the protocol, modifications will only applies to future inbound
    /// >           substreams, not the ones already being negotiated.
    pub fn listen_protocol_ref(&self) -> &SubstreamProtocol<P2PProtocol> {
        &self.listen_protocol
    }

    /// Returns a mutable reference to the listen protocol configuration.
    ///
    /// > **Note**: If you modify the protocol, modifications will only applies to future inbound
    /// >           substreams, not the ones already being negotiated.
    pub fn listen_protocol_mut(&mut self) -> &mut SubstreamProtocol<P2PProtocol> {
        &mut self.listen_protocol
    }

    /// Opens an outbound substream with a request.
    pub fn send_request(&mut self, p2p_event: P2PEvent) {
        self.keep_alive = KeepAlive::Yes;

        self.dial_queue.push(p2p_event);
    }
}

impl<TSubstream> ProtocolsHandler for P2PHandler<TSubstream>
where
    TSubstream: AsyncRead + AsyncWrite,
{
    type InEvent = P2PEvent;
    type OutEvent = P2PEvent;
    type Error = ProtocolsHandlerUpgrErr<P2PError>;
    type Substream = TSubstream;
    type InboundProtocol = P2PProtocol;
    type OutboundProtocol = P2PRequest;
    type OutboundOpenInfo = P2PEvent; // Keep track of the id and the request

    fn listen_protocol(&self) -> SubstreamProtocol<Self::InboundProtocol> {
        self.listen_protocol.clone()
    }

    fn inject_fully_negotiated_inbound(
        &mut self,
        out: <P2PProtocol as InboundUpgrade<TSubstream>>::Output,
    ) {
        // update the keep alive timeout if there are no more remaining outbound streams
        if let KeepAlive::Until(_) = self.keep_alive {
            self.keep_alive = KeepAlive::Until(Instant::now() + self.inactive_timeout);
        }

        let (req, substream) = out;
        // drop the stream and return a 0 id for goodbye "requests"
        if let r @ P2PRequest::Goodbye(_) = req {
            self.events_out.push(P2PEvent::Request(0, r));
            return;
        }

        // New inbound request. Store the stream and tag the output.
        let delay_key = self.inbound_substreams_delay.insert(
            self.current_inbound_substream_id,
            Duration::from_secs(RESPONSE_TIMEOUT),
        );
        let awaiting_stream = InboundSubstreamState::ResponseIdle(substream);
        self.inbound_substreams.insert(
            self.current_inbound_substream_id,
            (awaiting_stream, Some(delay_key)),
        );

        self.events_out
            .push(P2PEvent::Request(self.current_inbound_substream_id, req));
        self.current_inbound_substream_id += 1;
    }

    fn inject_fully_negotiated_outbound(
        &mut self,
        out: <P2PRequest as OutboundUpgrade<TSubstream>>::Output,
        p2p_event: Self::OutboundOpenInfo,
    ) {
        self.dial_negotiated -= 1;

        if self.dial_negotiated == 0
            && self.dial_queue.is_empty()
            && self.outbound_substreams.is_empty()
        {
            self.keep_alive = KeepAlive::Until(Instant::now() + self.inactive_timeout);
        } else {
            self.keep_alive = KeepAlive::Yes;
        }

        // add the stream to substreams if we expect a response, otherwise drop the stream.
        match p2p_event {
            P2PEvent::Request(id, request) if request.expect_response() => {
                // new outbound request. Store the stream and tag the output.
                let delay_key = self
                    .outbound_substreams_delay
                    .insert(id, Duration::from_secs(RESPONSE_TIMEOUT));
                let awaiting_stream = OutboundSubstreamState::RequestPendingResponse {
                    substream: out,
                    request,
                };
                if let Some(_) = self
                    .outbound_substreams
                    .insert(id, (awaiting_stream, delay_key))
                {
                    warn!(self.log, "Duplicate outbound substream id"; "id" => format!("{:?}", id));
                }
            }
            _ => { // a response is not expected, drop the stream for all other requests
            }
        }
    }

    // Note: If the substream has closed due to inactivity, or the substream is in the
    // wrong state a response will fail silently.
    fn inject_event(&mut self, rpc_event: Self::InEvent) {
        match rpc_event {
            P2PEvent::Request(_, _) => self.send_request(rpc_event),
            P2PEvent::Response(rpc_id, response) => {
                // check if the stream matching the response still exists
                // variables indicating if the response is an error response or a multi-part
                // response
                let res_is_error = response.is_error();
                let res_is_multiple = response.multiple_responses();

                match self.inbound_substreams.get_mut(&rpc_id) {
                    Some((substream_state, _)) => {
                        match std::mem::replace(substream_state, InboundSubstreamState::Poisoned) {
                            InboundSubstreamState::ResponseIdle(substream) => {
                                // close the stream if there is no response
                                if let P2PErrorResponse::StreamTermination(_) = response {
                                    //trace!(self.log, "Stream termination sent. Ending the stream");
                                    *substream_state = InboundSubstreamState::Closing(substream);
                                } else {
                                    // send the response
                                    // if it's a single p2p request or an error, close the stream after
                                    *substream_state = InboundSubstreamState::ResponsePendingSend {
                                        substream: substream.send(response),
                                        closing: !res_is_multiple | res_is_error, // close if an error or we are not expecting more responses
                                    };
                                }
                            }
                            InboundSubstreamState::ResponsePendingSend { substream, closing }
                                if res_is_multiple =>
                            {
                                // the stream is in use, add the request to a pending queue
                                self.queued_outbound_items
                                    .entry(rpc_id)
                                    .or_insert_with(Vec::new)
                                    .push(response);

                                // return the state
                                *substream_state = InboundSubstreamState::ResponsePendingSend {
                                    substream,
                                    closing,
                                };
                            }
                            InboundSubstreamState::Closing(substream) => {
                                *substream_state = InboundSubstreamState::Closing(substream);
                                debug!(self.log, "Response not sent. Stream is closing"; "response" => format!("{}",response));
                            }
                            InboundSubstreamState::ResponsePendingSend { substream, .. } => {
                                *substream_state = InboundSubstreamState::ResponsePendingSend {
                                    substream,
                                    closing: true,
                                };
                                error!(self.log, "Attempted sending multiple responses to a single response request");
                            }
                            InboundSubstreamState::Poisoned => {
                                crit!(self.log, "Poisoned inbound substream");
                                unreachable!("Coding error: Poisoned substream");
                            }
                        }
                    }
                    None => {
                        debug!(self.log, "Stream has expired. Response not sent"; "response" => format!("{}",response));
                    }
                };
            }
            // We do not send errors as responses
            P2PEvent::Error(_, _) => {}
        }
    }

    fn inject_dial_upgrade_error(
        &mut self,
        request: Self::OutboundOpenInfo,
        error: ProtocolsHandlerUpgrErr<
            <Self::OutboundProtocol as OutboundUpgrade<Self::Substream>>::Error,
        >,
    ) {
        if let ProtocolsHandlerUpgrErr::Upgrade(UpgradeError::Apply(P2PError::IoError(_))) = error {
            self.outbound_io_error_retries += 1;
            if self.outbound_io_error_retries < IO_ERROR_RETRIES {
                self.send_request(request);
                return;
            }
        }
        self.outbound_io_error_retries = 0;
        // add the error
        let request_id = {
            if let P2PEvent::Request(id, _) = request {
                id
            } else {
                0
            }
        };
        self.pending_error.push((request_id, error));
    }

    fn connection_keep_alive(&self) -> KeepAlive {
        self.keep_alive
    }

    fn poll(
        &mut self,
    ) -> Poll<
        ProtocolsHandlerEvent<Self::OutboundProtocol, Self::OutboundOpenInfo, Self::OutEvent>,
        Self::Error,
    > {
        if let Some((request_id, err)) = self.pending_error.pop() {
            // Returning an error here will result in dropping the peer.
            match err {
                ProtocolsHandlerUpgrErr::Upgrade(UpgradeError::Apply(
                                                     P2PError::InvalidProtocol(protocol_string),
                )) => {
                    // Peer does not support the protocol.
                    // TODO: We currently will not drop the peer, for maximal compatibility with
                    // other clients testing their software. In the future, we will need to decide
                    // which protocols are a bare minimum to support before kicking the peer.
                    error!(self.log, "Peer doesn't support the P2P protocol"; "protocol" => protocol_string);
                    return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                        P2PEvent::Error(request_id, P2PError::InvalidProtocol(protocol_string)),
                    )));
                }
                ProtocolsHandlerUpgrErr::Timeout | ProtocolsHandlerUpgrErr::Timer => {
                    // negotiation timeout, mark the request as failed
                    debug!(self.log, "Active substreams before timeout"; "len" => self.outbound_substreams.len());
                    return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                        P2PEvent::Error(
                            request_id,
                            P2PError::Custom("Protocol negotiation timeout".into()),
                        ),
                    )));
                }
                ProtocolsHandlerUpgrErr::Upgrade(UpgradeError::Apply(err)) => {
                    // IO/Decode/Custom Error, report to the application
                    return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                        P2PEvent::Error(request_id, err),
                    )));
                }
                ProtocolsHandlerUpgrErr::Upgrade(UpgradeError::Select(err)) => {
                    // Error during negotiation
                    return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                        P2PEvent::Error(request_id, P2PError::Custom(format!("{}", err))),
                    )));
                }
            }
        }

        // return any events that need to be reported
        if !self.events_out.is_empty() {
            return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                self.events_out.remove(0),
            )));
        } else {
            self.events_out.shrink_to_fit();
        }

        // purge expired inbound substreams and send an error
        while let Async::Ready(Some(stream_id)) = self
            .inbound_substreams_delay
            .poll()
            .map_err(|_| ProtocolsHandlerUpgrErr::Timer)?
        {
            let rpc_id = stream_id.get_ref();

            // handle a stream timeout for various states
            if let Some((substream_state, delay_key)) = self.inbound_substreams.get_mut(rpc_id) {
                // the delay has been removed
                *delay_key = None;

                let outbound_queue = self
                    .queued_outbound_items
                    .entry(*rpc_id)
                    .or_insert_with(Vec::new);
                substream_state.close(outbound_queue);
            }
        }

        // purge expired outbound substreams
        if let Async::Ready(Some(stream_id)) = self
            .outbound_substreams_delay
            .poll()
            .map_err(|_| ProtocolsHandlerUpgrErr::Timer)?
        {
            self.outbound_substreams.remove(stream_id.get_ref());
            // notify the user
            return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                P2PEvent::Error(
                    *stream_id.get_ref(),
                    P2PError::Custom("Stream timed out".into()),
                ),
            )));
        }

        // drive inbound streams that need to be processed
        for request_id in self.inbound_substreams.keys().copied().collect::<Vec<_>>() {
            // Drain all queued items until all messages have been processed for this stream
            // TODO Improve this code logic
            let mut new_items_to_send = true;
            while new_items_to_send {
                new_items_to_send = false;
                match self.inbound_substreams.entry(request_id) {
                    Entry::Occupied(mut entry) => {
                        match std::mem::replace(
                            &mut entry.get_mut().0,
                            InboundSubstreamState::Poisoned,
                        ) {
                            InboundSubstreamState::ResponsePendingSend {
                                mut substream,
                                closing,
                            } => {
                                match substream.poll() {
                                    Ok(Async::Ready(raw_substream)) => {
                                        // completed the send

                                        // close the stream if required
                                        if closing {
                                            entry.get_mut().0 =
                                                InboundSubstreamState::Closing(raw_substream)
                                        } else {
                                            // check for queued chunks and update the stream
                                            entry.get_mut().0 = apply_queued_responses(
                                                raw_substream,
                                                &mut self
                                                    .queued_outbound_items
                                                    .get_mut(&request_id),
                                                &mut new_items_to_send,
                                            );
                                        }
                                    }
                                    Ok(Async::NotReady) => {
                                        entry.get_mut().0 =
                                            InboundSubstreamState::ResponsePendingSend {
                                                substream,
                                                closing,
                                            };
                                    }
                                    Err(e) => {
                                        if let Some(delay_key) = &entry.get().1 {
                                            self.inbound_substreams_delay.remove(delay_key);
                                        }
                                        entry.remove_entry();
                                        return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                            P2PEvent::Error(0, e),
                                        )));
                                    }
                                };
                            }
                            InboundSubstreamState::ResponseIdle(substream) => {
                                entry.get_mut().0 = apply_queued_responses(
                                    substream,
                                    &mut self.queued_outbound_items.get_mut(&request_id),
                                    &mut new_items_to_send,
                                );
                            }
                            InboundSubstreamState::Closing(mut substream) => {
                                match substream.close() {
                                    Ok(Async::Ready(())) | Err(_) => {
                                        //trace!(self.log, "Inbound stream dropped");
                                        if let Some(delay_key) = &entry.get().1 {
                                            self.inbound_substreams_delay.remove(delay_key);
                                        }
                                        self.queued_outbound_items.remove(&request_id);
                                        entry.remove();

                                        if self.outbound_substreams.is_empty()
                                            && self.inbound_substreams.is_empty()
                                        {
                                            self.keep_alive = KeepAlive::Until(
                                                Instant::now() + self.inactive_timeout,
                                            );
                                        }
                                    } // drop the stream
                                    Ok(Async::NotReady) => {
                                        entry.get_mut().0 =
                                            InboundSubstreamState::Closing(substream);
                                    }
                                }
                            }
                            InboundSubstreamState::Poisoned => {
                                crit!(self.log, "Poisoned outbound substream");
                                unreachable!("Coding Error: Inbound Substream is poisoned");
                            }
                        };
                    }
                    Entry::Vacant(_) => unreachable!(),
                }
            }
        }

        // drive outbound streams that need to be processed
        for request_id in self.outbound_substreams.keys().copied().collect::<Vec<_>>() {
            match self.outbound_substreams.entry(request_id) {
                Entry::Occupied(mut entry) => {
                    match std::mem::replace(
                        &mut entry.get_mut().0,
                        OutboundSubstreamState::Poisoned,
                    ) {
                        OutboundSubstreamState::RequestPendingResponse {
                            mut substream,
                            request,
                        } => match substream.poll() {
                            Ok(Async::Ready(Some(response))) => {
                                if request.multiple_responses() && !response.is_error() {
                                    entry.get_mut().0 =
                                        OutboundSubstreamState::RequestPendingResponse {
                                            substream,
                                            request,
                                        };
                                    let delay_key = &entry.get().1;
                                    self.outbound_substreams_delay
                                        .reset(delay_key, Duration::from_secs(RESPONSE_TIMEOUT));
                                } else {
                                    // either this is a single response request or we received an
                                    // error
                                    //trace!(self.log, "Closing single stream request");
                                    // only expect a single response, close the stream
                                    entry.get_mut().0 = OutboundSubstreamState::Closing(substream);
                                }

                                return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                    P2PEvent::Response(request_id, response),
                                )));
                            }
                            Ok(Async::Ready(None)) => {
                                // stream closed
                                // if we expected multiple streams send a stream termination,
                                // else report the stream terminating only.
                                //trace!(self.log, "P2P Response - stream closed by remote");
                                // drop the stream
                                let delay_key = &entry.get().1;
                                self.outbound_substreams_delay.remove(delay_key);
                                entry.remove_entry();
                                // notify the application error
                                if request.multiple_responses() {
                                    // return an end of stream result
                                    return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                        P2PEvent::Response(
                                            request_id,
                                            P2PErrorResponse::StreamTermination(
                                                request.stream_termination(),
                                            ),
                                        ),
                                    )));
                                } // else we return an error, stream should not have closed early.
                                return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                    P2PEvent::Error(
                                        request_id,
                                        P2PError::Custom(
                                            "Stream closed early. Empty response".into(),
                                        ),
                                    ),
                                )));
                            }
                            Ok(Async::NotReady) => {
                                entry.get_mut().0 = OutboundSubstreamState::RequestPendingResponse {
                                    substream,
                                    request,
                                }
                            }
                            Err(e) => {
                                // drop the stream
                                let delay_key = &entry.get().1;
                                self.outbound_substreams_delay.remove(delay_key);
                                entry.remove_entry();
                                return Ok(Async::Ready(ProtocolsHandlerEvent::Custom(
                                    P2PEvent::Error(request_id, e),
                                )));
                            }
                        },
                        OutboundSubstreamState::Closing(mut substream) => match substream.close() {
                            Ok(Async::Ready(())) | Err(_) => {
                                //trace!(self.log, "Outbound stream dropped");
                                // drop the stream
                                let delay_key = &entry.get().1;
                                self.outbound_substreams_delay.remove(delay_key);
                                entry.remove_entry();

                                if self.outbound_substreams.is_empty()
                                    && self.inbound_substreams.is_empty()
                                {
                                    self.keep_alive =
                                        KeepAlive::Until(Instant::now() + self.inactive_timeout);
                                }
                            }
                            Ok(Async::NotReady) => {
                                entry.get_mut().0 = OutboundSubstreamState::Closing(substream);
                            }
                        },
                        OutboundSubstreamState::Poisoned => {
                            crit!(self.log, "Poisoned outbound substream");
                            unreachable!("Coding Error: Outbound substream is poisoned")
                        }
                    }
                }
                Entry::Vacant(_) => unreachable!(),
            }
        }

        // establish outbound substreams
        if !self.dial_queue.is_empty() && self.dial_negotiated < self.max_dial_negotiated {
            self.dial_negotiated += 1;
            let rpc_event = self.dial_queue.remove(0);
            self.dial_queue.shrink_to_fit();
            if let P2PEvent::Request(id, req) = rpc_event {
                return Ok(Async::Ready(
                    ProtocolsHandlerEvent::OutboundSubstreamRequest {
                        protocol: SubstreamProtocol::new(req.clone()),
                        info: P2PEvent::Request(id, req),
                    },
                ));
            }
        }
        Ok(Async::NotReady)
    }
}

// Check for new items to send to the peer and update the underlying stream
fn apply_queued_responses<TSubstream: AsyncRead + AsyncWrite>(
    raw_substream: InboundFramed<TSubstream>,
    queued_outbound_items: &mut Option<&mut Vec<P2PErrorResponse>>,
    new_items_to_send: &mut bool,
) -> InboundSubstreamState<TSubstream> {
    match queued_outbound_items {
        Some(ref mut queue) if !queue.is_empty() => {
            *new_items_to_send = true;
            // we have queued items
            match queue.remove(0) {
                P2PErrorResponse::StreamTermination(_) => {
                    // close the stream if this is a stream termination
                    InboundSubstreamState::Closing(raw_substream)
                }
                chunk => InboundSubstreamState::ResponsePendingSend {
                    substream: raw_substream.send(chunk),
                    closing: false,
                },
            }
        }
        _ => {
            // no items queued set to idle
            InboundSubstreamState::ResponseIdle(raw_substream)
        }
    }
}
