use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum API {
    Chain,
    TxPool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub modules: Vec<API>,
}

impl Config {
    pub fn config_chain(&self) -> bool {
        self.modules.contains(&API::Chain)
    }

    pub fn config_pool(&self) -> bool {
        self.modules.contains(&API::TxPool)
    }
}
