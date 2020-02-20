#[cfg(test)]
mod tests {
    use jsonrpc_http_server::jsonrpc_core::*;
    #[test]
    fn test_handler() {
        let mut io = IoHandler::new();
        io.add_method("getVersion", |_: Params| Ok(Value::String("1.0".to_owned())));

        let request = r#"{"jsonrpc": "2.0", "method": "getVersion", "params": [0], "id": 1}"#;
        let response = r#"{"jsonrpc":"2.0","result":"1.0","id":1}"#;

        assert_eq!(io.handle_request_sync(request), Some(response.to_owned()));
    }
}