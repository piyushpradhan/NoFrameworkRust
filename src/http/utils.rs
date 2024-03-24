use dotenv::dotenv;
use jsonwebtoken::{decode, errors::Error, Algorithm, DecodingKey, TokenData, Validation};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::to_string;

use crate::app::{
    models::claims::Claims,
    services::utils::{generate_token, verify_token},
};

fn http_status_text(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        401 => "Unauthorized",
        403 => "Access Denied",
        500 => "Internal Server Error",
        _ => "Unknown status",
    }
}

pub fn generate_http_response<T: serde::Serialize>(status_code: u16, data: &T) -> String {
    let response = to_string(data).unwrap();

    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: http://localhost:8080\r\nAccess-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE\r\nAccess-Control-Allow-Credentials: true\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        status_code,
        http_status_text(status_code),
        response.len(),
        response
    )
}

pub fn generate_options_response() -> String {
    let response = "";

    format!(
        "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: http://localhost:8080\r\nAccess-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE\r\nAccess-Control-Allow-Headers: content-type, Authorization, withCredentials, Cookie\r\nAccess-Control-Max-Age: 86400\r\nContent-Length: {}\r\n\r\n{}",
        response.len(),
        response
    )
}

pub fn not_found_response() -> String {
    let response = String::from("This route does not exist");

    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Credentials: true\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        404,
        http_status_text(404),
        response.len(),
        response
    )
}

pub fn unauthorized_response(message: &str) -> String {
    let response = String::from(message);

    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        401,
        http_status_text(404),
        response.len(),
        response
    )
}

pub fn something_went_wrong(message: String) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        500,
        http_status_text(500),
        message.len(),
        message
    )
}

pub fn initial_sse_response() -> String {
    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: keep-alive\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\n\r\n"
        .to_string()
}

fn extract_uri(request: &str) -> String {
    let lines: Vec<&str> = request.lines().collect();
    if let Some(request_line) = lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return String::from(parts[1]);
        }
    }

    String::from("/")
}

fn extract_method(request: &str) -> String {
    let lines: Vec<&str> = request.lines().collect();
    if let Some(request_line) = lines.first() {
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() >= 2 {
            return String::from(parts[0]);
        }
    }
    String::from("GET")
}

fn extract_auth_header(request: &str) -> Option<&str> {
    for line in request.lines() {
        if let Some(header_value) = line.strip_prefix("authorization: ") {
            return Some(header_value);
        }
    }

    None
}

fn extract_cookies(request: &str) -> Option<Vec<(&str, &str)>> {
    let mut cookies = vec![];

    for line in request.lines() {
        if let Some(cookie_line) = line.strip_prefix("Cookie: ") {
            for cookie in cookie_line.split(';') {
                let mut parts = cookie.trim().split('=');
                if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
                    cookies.push((name, value));
                }
            }
        }
    }

    if cookies.is_empty() {
        None
    } else {
        Some(cookies)
    }
}

fn extract_body(request: &str) -> String {
    let lines: Vec<&str> = request.lines().collect();

    // Find the index of the string where the body of the request starts
    // Basically we're looking for an empty line
    let body_start_index = lines
        .iter()
        .position(|line| line.is_empty() && line.chars().all(|c| c.is_whitespace()));

    // The body of the request is from body_start_index to the end of the request
    match body_start_index {
        Some(start) if start + 1 < lines.len() => lines[start + 1..].join("\n"),
        _ => String::new(),
    }
}

pub fn extract_request(
    request: &str,
) -> (
    String,
    String,
    Option<&str>,
    String,
    Option<Vec<(&str, &str)>>,
) {
    let uri = extract_uri(request);
    let method = extract_method(request);
    let authorization_header = extract_auth_header(request);
    let body = extract_body(request);
    let cookies = extract_cookies(request);

    (uri, method, authorization_header, body, cookies)
}

pub fn should_require_token_verification(url: &str) -> bool {
    let public_routes = vec!["/auth"];
    !public_routes.iter().any(|route| url.starts_with(route))
}

pub fn is_token_expired(access_token: &str, cookies: Option<Vec<(&str, &str)>>) -> bool {
    // Check if the token is not empty
    if access_token.is_empty() {
        return true;
    }

    let decoded_token = verify_token(&access_token, cookies);

    // Get the current time as a Unix timestamp
    let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs() as usize,
        Err(_) => return true, // If the current time is before UNIX_EPOCH, return false
    };

    // Compare the current time with the expiration time
    current_time > decoded_token.unwrap().claims.exp
}

pub fn extract_token_from_cookies(cookies: Option<Vec<(&str, &str)>>) -> Option<String> {
    let token = cookies.and_then(|cookies| {
        cookies
            .iter()
            .find(|(name, _)| *name == "token")
            .map(|(_, value)| value.to_string())
    });
    token
}

pub fn access_token_from_refresh(refresh_token: &str) -> Option<String> {
    let decoded = verify_refresh_token(&refresh_token);
    match decoded {
        Ok(refresh_data) => {
            let claims = refresh_data.claims;
            let access_token = generate_token(claims.uid, claims.username.as_str());
            Some(access_token)
        }
        _ => None,
    }
}

pub fn refresh_access_token(cookies: Option<Vec<(&str, &str)>>) -> Result<String, bool> {
    let refresh_token = &cookies.as_ref().and_then(|cookies| {
        cookies
            .iter()
            .find(|(name, _)| *name == "refresh")
            .map(|(_, value)| value.to_string())
    });

    match refresh_token {
        Some(token) => {
            let is_expired = is_token_expired(&token, cookies.clone());
            if is_expired {
                return Err(false);
            }

            let access_token = access_token_from_refresh(&token);

            match access_token {
                Some(access_token) => return Ok(access_token),
                _ => return Err(false),
            }
        }
        None => return Err(false),
    }
}

pub fn verify_refresh_token(token: &str) -> Result<TokenData<Claims>, Error> {
    dotenv().ok();

    let secret = match env::var("REFRESH_TOKEN_SECRET") {
        Ok(refresh_secret) => refresh_secret,
        _ => panic!("REFRESH_TOKEN_SECRET is not provided"),
    };

    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => Ok(token_data),
        Err(error) => {
            return Err(error);
        }
    }
}

pub fn extract_token_from_auth(auth_header: &str) -> Vec<(&str, &str)> {
    let pairs: Vec<(&str, &str)> = auth_header
        .trim_start_matches("Bearer ")
        .split(';')
        .map(|pair| {
            let mut iter = pair.split('=');
            (iter.next().unwrap(), iter.next().unwrap_or(""))
        })
        .collect();

    return pairs;
}
