#[macro_use]
extern crate log;

pub mod handler;
pub mod transport;
pub mod behaviour;
pub mod config;

pub use config::{Config as NetworkConfig};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
