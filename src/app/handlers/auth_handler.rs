use sqlx::Error as PgError;
use std::env;
use std::sync::mpsc::Sender;

use crate::{
    app::{models::user::User, services::auth::AuthService},
    http::utils::{generate_http_response, not_found_response, something_went_wrong},
};
use serde_json::{Error, Value};

async fn setup() -> Result<AuthService, PgError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not specified");
    let pool = sqlx::postgres::PgPool::connect(database_url.as_str()).await?;
    let cloned_pool = pool.clone();
    let auth_service = AuthService::new(pool);

    Ok(auth_service)
}

fn parse_json(json_string: &str) -> Result<Value, Error> {
    let parsed_json: Result<Value, Error> = serde_json::from_str(json_string);
    parsed_json
}

pub async fn login(
    sender: &Sender<String>,
    body: String,
    cookies: Option<Vec<(&str, &str)>>,
) -> String {
    match setup().await {
        Ok(auth_service) => {
            let username;
            let password;

            match parse_json(body.as_str()) {
                Ok(data) => {
                    username = data["username"].to_string();
                    password = data["password"].to_string();
                }
                Err(err) => {
                    username = String::new();
                    password = String::new();
                }
            }

            let response: Result<User, PgError> = auth_service
                .login(username.as_str(), password.as_str(), cookies)
                .await;

            match response {
                Ok(response) => {
                    let formatted_response = generate_http_response(200, &response);

                    let _ = sender.send(formatted_response.clone());
                    return formatted_response;
                }
                Err(error) => something_went_wrong(error.to_string()),
            }
        }
        _ => {
            return not_found_response();
        }
    };

    return not_found_response();
}

pub async fn register(sender: &Sender<String>, body: String) -> String {
    match setup().await {
        Ok(auth_service) => {
            let username;
            let password;

            match parse_json(body.as_str()) {
                Ok(data) => {
                    username = data["username"].to_string();
                    password = data["password"].to_string();
                }
                Err(_) => {
                    username = String::new();
                    password = String::new();
                }
            }

            let response: Result<User, PgError> = auth_service.register(&username, &password).await;

            match response {
                Ok(response) => {
                    let formatted_response = generate_http_response(200, &response);

                    let _ = sender.send(formatted_response.clone());
                    return formatted_response;
                }
                Err(error) => something_went_wrong(error.to_string()),
            }
        }
        _ => {
            return not_found_response();
        }
    };

    return not_found_response();
}
