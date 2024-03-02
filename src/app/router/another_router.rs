use std::sync::mpsc;

use crate::{app::handlers::test_handler::test_api, http::utils::not_found_response};

pub struct AnotherRouter {
    sender: mpsc::Sender<String>,
}

impl AnotherRouter {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        AnotherRouter { sender }
    }

    pub fn route(
        &self,
        method: &str,
        path: &str,
        _authorization_header: Option<&str>,
        _body: String,
    ) -> String {
        if !path.starts_with("/another") {
            return not_found_response();
        }

        match (method, path) {
            ("GET", "/another") => test_api(self.sender.clone()),
            ("POST", "/another/create") => test_api(self.sender.clone()),
            _ => not_found_response(),
        }
    }
}
