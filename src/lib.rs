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
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: u64, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap_or_else(|e| {
                // println!("{}", e);
                panic!("{}", e);
            });
            println!("Worker {} got a job; executing.", id);
            job.call_box();
        });
        Worker { id, thread }
    }
}

#[derive(Debug)]
pub struct PoolCreationErr;

type Job = Box<dyn FnBox + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
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

        Ok(ThreadPool { workers, sender })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
