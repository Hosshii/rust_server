use std::thread;

pub struct ThreadPool {
    threads: Vec<thread::JoinHandle<()>>,
}

#[derive(Debug)]
pub struct PoolCreationErr;

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

        let mut threads = Vec::with_capacity(size);
        for _ in 0..size {}
        Ok(ThreadPool { threads })
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}
