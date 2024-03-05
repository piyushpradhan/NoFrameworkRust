use std::sync::mpsc;

use crate::{
    app::handlers::{auth_handler::login, test_handler::test_api},
    http::utils::not_found_response,
};

pub struct AuthRouter {
    sender: mpsc::Sender<String>,
}

impl AuthRouter {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        AuthRouter { sender }
    }

    pub async fn route(
        &self,
        method: &str,
        path: &str,
        _authorization_header: Option<&str>,
        body: String,
    ) -> String {
        if !path.starts_with("/auth") {
            return not_found_response();
        }

        match (method, path) {
            ("POST", "/auth/login") => login(&self.sender, body).await,
            ("POST", "/auth/register") => test_api(self.sender.clone()),
            _ => not_found_response(),
        }
    }
}
