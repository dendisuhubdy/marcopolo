pub mod handler;
pub mod transport;
pub mod behaviour;
pub mod config;
pub mod executor;
pub mod error;

pub use libp2p::gossipsub::{Topic, TopicHash};
pub use libp2p::multiaddr;
pub use libp2p::{
    gossipsub::{GossipsubConfig, GossipsubConfigBuilder},
    PeerId,
};
pub use config::{Config as NetworkConfig};
pub use libp2p::multiaddr::Multiaddr;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
