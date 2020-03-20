use std::io;

use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_core::Result;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::{Server, ServerBuilder};

pub fn start_http(ip: String, port: u16) {
    let url = format!("{}:{}", ip, port);
    let addr = url.parse().map_err(|_| format!("Invalid  listen host/port given: {}", url)).unwrap();

    let mut io = IoHandler::new();
    io.add_method("send_transaction", |_params: Params| {
        Ok(Value::String("transaction".to_string()))
    });

    let server = ServerBuilder::new(io)
        .threads(4)
        .start_http(&addr)
        .unwrap();

    server.wait();
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