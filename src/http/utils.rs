use serde_json::to_string;

fn http_status_text(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        500 => "Internal Server Error",
        _ => "Unknown status",
    }
}

pub fn generate_http_response<T: serde::Serialize>(status_code: u16, data: &T) -> String {
    let response = to_string(data).unwrap();

    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        status_code,
        http_status_text(status_code),
        response.len(),
        response
    )
}

pub fn not_found_response() -> String {
    let response = String::from("This route does not exist");

    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        404,
        http_status_text(404),
        response.len(),
        response
    )
}

pub fn something_went_wrong(message: String) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\n\r\n{}",
        404,
        http_status_text(404),
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
        if let Some(header_value) = line.strip_prefix("Authorization: ") {
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
