use std::net::TcpStream;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

struct Worker {
    id: u64,
    thread: Option<thread::JoinHandle<()>>,
    stream: Option<TcpStream>,
}

impl Worker {
    fn new(id: u64, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .unwrap_or_else(|e| {
                    panic!("lock err: {}", e);
                })
                .recv()
                .unwrap_or_else(|e| {
                    // println!("{}", e);
                    panic!("receive err: {}", e);
                });

            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job.call_box();
                }
                Message::Terminate => {
                    println!("Worker {} was told to terminate", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
            stream: None,
        }
    }
}

#[derive(Debug)]
pub struct PoolCreationErr;

type Job = Box<dyn FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Arc<Mutex<mpsc::Sender<Message>>>,
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationErr> {
        if size <= 0 {
            return Err(PoolCreationErr);
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for i in 0..size {
            workers.push(Worker::new(i as u64, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Arc::new(Mutex::new(sender)),
        })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender
            .lock()
            .unwrap()
            .send(Message::NewJob(job))
            .unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Setting terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender
                .lock()
                .unwrap()
                .send(Message::Terminate)
                .unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
