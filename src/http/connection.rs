use dotenv::dotenv;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
    sync::mpsc,
    thread,
};

use crate::app::router::app::Router;

use super::utils::{
    extract_request, extract_token_from_auth, extract_token_from_cookies, is_token_expired,
    refresh_access_token, should_require_token_verification, unauthorized_response,
};

fn handle_options_request(stream: &mut TcpStream) {
    let response = "HTTP/1.1 200 OK\r\n\
    Access-Control-Allow-Origin: *\r\n\
    Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS\r\n\
    Access-Control-Allow-Headers: Content-Type, Authorization, Cookie\r\n\
    Content-Length: 0\r\n\
    \r\n";

    if let Err(err) = stream.write_all(response.as_bytes()) {
        eprintln!("Error writing OPTIONS response to stream: {:?}", err);
    }
}

pub async fn handle_connection(mut stream: TcpStream) {
    dotenv().ok();

    let mut buffer = Vec::new();

    loop {
        // Read in chunks of 1024 bytes
        let mut local_buf = vec![0; 1024];

        // Check if the number of bytes being read is greater than 0
        // i.e. data has been read successfully
        let bytes_read = match stream.read(&mut local_buf) {
            Ok(n) if n > 0 => n,
            _ => break,
        };

        // Append the read bytes to the resultant vector
        buffer.extend_from_slice(&local_buf[..bytes_read]);

        // Line breaks
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&buffer);
    let (uri, method, authorization_header, body, cookies) = extract_request(&request);

    let mut request_cookies = cookies.clone();
    let session_token = cookies
        .clone()
        .unwrap_or_else(|| extract_token_from_auth(authorization_header.unwrap_or_else(|| "")));

    // NOTE: Add the public routes in this function
    let is_token_verification_required = should_require_token_verification(&uri);

    if method == "OPTIONS" {
        handle_options_request(&mut stream);
        return;
    }

    // Verify access token before moving forward
    if is_token_verification_required {
        match extract_token_from_cookies(Some(session_token.clone())) {
            Some(access_token) => {
                if is_token_expired(&access_token, Some(session_token.clone())) {
                    let access_token = refresh_access_token(Some(session_token.clone()));

                    match access_token {
                        Ok(token) => {
                            if let Some(modified_cookies) = request_cookies.as_mut() {
                                for (key, value) in modified_cookies.iter_mut() {
                                    if *key == "token" {
                                        *value = &token;
                                    }
                                }
                            }
                        }
                        _ => {
                            let response = unauthorized_response("Could not verify access token");
                            stream.write(response.as_bytes()).unwrap();
                            stream.flush().unwrap();
                            return;
                        }
                    }
                }
            }
            None => {
                let response = unauthorized_response("Could not extract access token");
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
                return;
            }
        }
    }

    // For communicating between threads
    let (sender, receiver) = mpsc::channel::<String>();

    let app_router: Router = Router::new(sender.clone());
    let response = app_router
        .route(
            method.as_str(),
            uri.as_str(),
            authorization_header,
            body,
            Some(session_token),
        )
        .await;

    // Write the response to stream
    let _write = stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    thread::spawn(move || {
        // Reading data that's being sent to the receiver
        for data in receiver {
            if let Err(err) = stream.write(data.as_bytes()) {
                eprintln!("Error writing to stream: {:?}", err);
                // Attempt to gracefully close the stream and break the loop
                if let Err(close_err) = stream.shutdown(Shutdown::Both) {
                    eprintln!("Error shutting down stream: {:?}", close_err);
                }
                break;
            }
            if let Err(err) = stream.flush() {
                eprintln!("Error flushing stream: {:?}", err);
                // Attempt to gracefully close the stream and break the loop
                if let Err(close_err) = stream.shutdown(Shutdown::Both) {
                    eprintln!("Error shutting down stream after flushing: {:?}", close_err);
                }
                break;
            }
        }
    });
}
