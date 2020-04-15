use libp2p::{
    core::{either::EitherTransport, transport::upgrade::Version, StreamMuxer},
    identity,
    pnet::{PnetConfig, PreSharedKey},
    secio::SecioConfig,
    tcp::TcpConfig,
    yamux::Config as YamuxConfig,
    PeerId, Transport,
};
use std::{error::Error,time::Duration, };
use async_std::{io};

/// Builds the transport that serves as a common ground for all connections.
pub fn build_transport(
    key_pair: identity::Keypair,
    psk: Option<PreSharedKey>,
) -> impl Transport<
    Output = (
        PeerId,
        impl StreamMuxer<
            OutboundSubstream = impl Send,
            Substream = impl Send,
            Error = impl Into<io::Error>,
        > + Send
        + Sync,
    ),
    Error = impl Error + Send,
    Listener = impl Send,
    Dial = impl Send,
    ListenerUpgrade = impl Send,
> + Clone {
    let secio_config = SecioConfig::new(key_pair);
    let yamux_config = YamuxConfig::default();

    let base_transport = TcpConfig::new().nodelay(true);
    let maybe_encrypted = match psk {
        Some(psk) => EitherTransport::Left(
            base_transport.and_then(move |socket, _| PnetConfig::new(psk).handshake(socket)),
        ),
        None => EitherTransport::Right(base_transport),
    };
    maybe_encrypted
        .upgrade(Version::V1)
        .authenticate(secio_config)
        .multiplex(yamux_config)
        .timeout(Duration::from_secs(20))
}