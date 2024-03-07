use std::sync::mpsc;

use crate::app::handlers::test_handler::test_api;
use crate::http::utils::not_found_response;

use super::another_router::AnotherRouter;
use super::auth_router::AuthRouter;
use super::test_router::TestRouter;

pub struct Router {
    sender: mpsc::Sender<String>,
    test_router: TestRouter,
    another_router: AnotherRouter,
    auth_router: AuthRouter,
}

impl Router {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        let test_router = TestRouter::new(sender.clone());
        let another_router = AnotherRouter::new(sender.clone());
        let auth_router = AuthRouter::new(sender.clone());

        Router {
            sender,
            test_router,
            another_router,
            auth_router,
        }
    }

    pub async fn route(
        &self,
        method: &str,
        path: &str,
        authorization_header: Option<&str>,
        body: String,
        cookies: Option<Vec<(&str, &str)>>,
    ) -> String {
        let mut segments = path.trim_matches('/').split("/");
        let mut prefix = "/";

        if let Some(first_segment) = segments.next() {
            prefix = first_segment;
        }

        match prefix {
            "/" => test_api(self.sender.clone()),
            "test" => self
                .test_router
                .route(method, path, authorization_header, body),
            "another" => self
                .another_router
                .route(method, path, authorization_header, body),
            "auth" => {
                self.auth_router
                    .route(method, path, authorization_header, body, cookies)
                    .await
            }
            _ => not_found_response(),
        }
    }
}
