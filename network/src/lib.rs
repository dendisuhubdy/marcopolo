pub use libp2p::{
    gossipsub::{GossipsubConfig, GossipsubConfigBuilder},
    PeerId,
};
pub use libp2p::gossipsub::{Topic, TopicHash};
pub use libp2p::multiaddr;
pub use libp2p::multiaddr::Multiaddr;

pub use config::Config as NetworkConfig;

pub mod handler;
pub mod transport;
pub mod behaviour;
pub mod config;
pub mod executor;
pub mod error;
pub mod p2p;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
