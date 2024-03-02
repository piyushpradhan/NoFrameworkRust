use std::sync::mpsc;

use crate::{app::handlers::test_handler::test_api, http::utils::not_found_response};

pub struct TestRouter {
    sender: mpsc::Sender<String>,
}

impl TestRouter {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        TestRouter { sender }
    }

    pub fn route(
        &self,
        method: &str,
        path: &str,
        _authorization_header: Option<&str>,
        _body: String,
    ) -> String {
        if !path.starts_with("/test") {
            return not_found_response();
        }

        match (method, path) {
            ("GET", "/test") => test_api(self.sender.clone()),
            ("POST", "/test/create") => test_api(self.sender.clone()),
            _ => not_found_response(),
        }
    }
}
