use dotenv::dotenv;
use http::connection;
use http::thread_pool::ThreadPool;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

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
    // let pool = Arc::new(ThreadPool::new(4));

    for stream in listener.incoming() {
        let stream_value = stream.unwrap();
        // let thread_pool = Arc::clone(&pool);

        // Execute the connection handling task within the thread pool
        // thread_pool.execute(async move {
        connection::handle_connection(stream_value).await;
        // });
    }
}
