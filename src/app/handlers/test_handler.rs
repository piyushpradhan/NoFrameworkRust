use std::sync::mpsc::Sender;

use crate::http::utils::generate_http_response;

pub fn test_api(sender: Sender<String>) -> String {
    let response: Option<String> = Some(String::from("This works"));
    let error_message = String::from("Something went wrong");

    let formatted_response = match response {
        Some(result) => generate_http_response(200, &result),
        None => generate_http_response(500, &error_message),
    };

    // Send the data to receiver
    sender.send(formatted_response.clone()).unwrap();

    formatted_response
}
