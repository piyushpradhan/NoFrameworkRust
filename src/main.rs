use dotenv::dotenv;
use http::connection::{self};
use http::thread_pool::ThreadPool;
use std::env::{self};
use std::net::TcpListener;

mod app;
mod http;

fn main() {
    dotenv().ok();

    let address = match env::var("APP_URL") {
        Ok(address) => address,
        _ => String::from("127.0.0.1:7878"),
    };

    let listener = TcpListener::bind(address).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream_value = stream.unwrap();

        // Sending the handle_connection job to the worker to execute
        // Basically each of those workers will keep listening to for
        // HTTP Requests
        pool.execute(|| {
            connection::handle_connection(stream_value);
        });
    }
}
