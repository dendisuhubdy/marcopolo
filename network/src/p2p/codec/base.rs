//! This handles the various supported encoding mechanism for the MAP P2P.

use crate::p2p::{ErrorMessage, P2PErrorResponse, P2PRequest, P2PResponse};
use libp2p::bytes::BufMut;
use libp2p::bytes::BytesMut;
use tokio::codec::{Decoder, Encoder};

pub trait OutboundCodec: Encoder + Decoder {
    type ErrorType;

    fn decode_error(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::ErrorType>, <Self as Decoder>::Error>;
}

/* Global Inbound Codec */
// This deals with Decoding P2P Requests from other peers and encoding our responses

pub struct BaseInboundCodec<TCodec>
where
    TCodec: Encoder + Decoder,
{
    /// Inner codec for handling various encodings
    inner: TCodec,
}

impl<TCodec> BaseInboundCodec<TCodec>
where
    TCodec: Encoder + Decoder,
{
    pub fn new(codec: TCodec) -> Self {
        BaseInboundCodec { inner: codec }
    }
}

/* Global Outbound Codec */
// This deals with Decoding P2P Responses from other peers and encoding our requests
pub struct BaseOutboundCodec<TOutboundCodec>
where
    TOutboundCodec: OutboundCodec,
{
    /// Inner codec for handling various encodings.
    inner: TOutboundCodec,
    /// Keeps track of the current response code for a chunk.
    current_response_code: Option<u8>,
}

impl<TOutboundCodec> BaseOutboundCodec<TOutboundCodec>
where
    TOutboundCodec: OutboundCodec,
{
    pub fn new(codec: TOutboundCodec) -> Self {
        BaseOutboundCodec {
            inner: codec,
            current_response_code: None,
        }
    }
}

/* Implementation of the Encoding/Decoding for the global codecs */

/* Base Inbound Codec */

// This Encodes P2P Responses sent to external peers
impl<TCodec> Encoder for BaseInboundCodec<TCodec>
where
    TCodec: Decoder + Encoder<Item =P2PErrorResponse>,
{
    type Item = P2PErrorResponse;
    type Error = <TCodec as Encoder>::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.clear();
        dst.reserve(1);
        dst.put_u8(
            item.as_u8()
                .expect("Should never encode a stream termination"),
        );
        self.inner.encode(item, dst)
    }
}

// This Decodes P2P Requests from external peers
impl<TCodec> Decoder for BaseInboundCodec<TCodec>
where
    TCodec: Encoder + Decoder<Item =P2PRequest>,
{
    type Item = P2PRequest;
    type Error = <TCodec as Decoder>::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.inner.decode(src)
    }
}

/* Base Outbound Codec */

// This Encodes P2P Requests sent to external peers
impl<TCodec> Encoder for BaseOutboundCodec<TCodec>
where
    TCodec: OutboundCodec + Encoder<Item =P2PRequest>,
{
    type Item = P2PRequest;
    type Error = <TCodec as Encoder>::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.inner.encode(item, dst)
    }
}

// This decodes P2P Responses received from external peers
impl<TCodec> Decoder for BaseOutboundCodec<TCodec>
where
    TCodec: OutboundCodec<ErrorType = ErrorMessage> + Decoder<Item =P2PResponse>,
{
    type Item = P2PErrorResponse;
    type Error = <TCodec as Decoder>::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // if we have only received the response code, wait for more bytes
        if src.len() <= 1 {
            return Ok(None);
        }
        // using the response code determine which kind of payload needs to be decoded.
        let response_code = self.current_response_code.unwrap_or_else(|| {
            let resp_code = src.split_to(1)[0];
            self.current_response_code = Some(resp_code);
            resp_code
        });

        let inner_result = {
            if P2PErrorResponse::is_response(response_code) {
                // decode an actual response and mutates the buffer if enough bytes have been read
                // returning the result.
                self.inner
                    .decode(src)
                    .map(|r| r.map(P2PErrorResponse::Success))
            } else {
                // decode an error
                self.inner
                    .decode_error(src)
                    .map(|r| r.map(|resp| P2PErrorResponse::from_error(response_code, resp)))
            }
        };
        // if the inner decoder was capable of decoding a chunk, we need to reset the current
        // response code for the next chunk
        if let Ok(Some(_)) = inner_result {
            self.current_response_code = None;
        }
        // return the result
        inner_result
    }
}
