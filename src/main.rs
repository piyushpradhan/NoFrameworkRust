use dotenv::dotenv;
use http::connection;
use http::thread_pool::ThreadPool;
use std::env;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use tokio::runtime::Handle;

mod app;
mod http;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let address = match env::var("APP_URL") {
        Ok(address) => address,
        _ => String::from("127.0.0.1:7878"),
    };

    let listener = TcpListener::bind(address).unwrap();

    // Make pool a shared resource that is in sync across threads
    let pg_pool = Arc::new(Mutex::new(ThreadPool::new(4)));

    for stream in listener.incoming() {
        let stream_value = stream.unwrap();
        let pg_pool = Arc::clone(&pg_pool);

        // Execute the connection handling task within the Tokio runtime
        Handle::current().spawn(async move {
            let cloned_stream_value = stream_value.try_clone().unwrap();
            connection::handle_connection(cloned_stream_value).await;
        });
    }
}
