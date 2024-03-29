use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            match message {
                Message::NewJob(job) => {
                    let _ = tokio::runtime::Runtime::new().unwrap().block_on(job);
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    // Creates a new thread pool
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "Size of the thread pool should be greater than 0");

        let (sender, channel_receiver) = mpsc::channel::<Message>();
        let receiver = Arc::new(Mutex::new(channel_receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    // Send the job to be executed
    pub fn execute<F>(&self, f: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let job = Box::pin(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Send termination message to all workers
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            // Wait for the associated threads to finish
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
