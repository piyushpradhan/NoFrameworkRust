use std::sync::mpsc;

use crate::{
    app::handlers::auth_handler::{login, register},
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
        cookies: Option<Vec<(&str, &str)>>,
    ) -> String {
        if !path.starts_with("/auth") {
            return not_found_response();
        }

        match (method, path) {
            ("POST", "/auth/login") => login(&self.sender, body, cookies).await,
            ("POST", "/auth/register") => register(&self.sender, body).await,
            _ => not_found_response(),
        }
    }
}
