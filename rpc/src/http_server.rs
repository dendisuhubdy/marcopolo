use std::sync::{Arc, RwLock};

use jsonrpc_http_server::{AccessControlAllowOrigin, DomainsValidation, RestApi, ServerBuilder};

use chain::blockchain::BlockChain;

use crate::rpc_build::RpcBuilder;

pub struct RpcServer {
    pub http: jsonrpc_http_server::Server,
    pub url: String,
}

pub fn start_http(ip: String, port: u16, block_chain: Arc<RwLock<BlockChain>>) -> RpcServer {
    let url = format!("{}:{}", ip, port);

    info!("using url {}", url);

    let addr = url.parse().map_err(|_| format!("Invalid  listen host/port given: {}", url)).unwrap();

    let handler = RpcBuilder::new().config_chain(block_chain).config_pool().build();

    let http = ServerBuilder::new(handler)
        .threads(4)
        .rest_api(RestApi::Unsecure)
        .cors(DomainsValidation::AllowOnly(vec![AccessControlAllowOrigin::Any]))
        .start_http(&addr)
        .expect("Start json rpc HTTP service failed");
    RpcServer { http, url }
}

impl RpcServer {
    pub fn close(self) {
        self.http.close();
        info!(" rpc http stop {} ", self.url);
    }
}

#[cfg(test)]
mod tests {
    use jsonrpc_core::*;

    #[test]
    fn test_handler() {
        let mut io = IoHandler::new();
        io.add_method("getVersion", |_: Params| Ok(Value::String("1.0".to_owned())));

        let request = r#"{"jsonrpc": "2.0", "method": "getVersion", "params": [0], "id": 1}"#;
        let response = r#"{"jsonrpc":"2.0","result":"1.0","id":1}"#;

        assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
    }
}