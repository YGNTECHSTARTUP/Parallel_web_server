use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam::channel::{Receiver, Sender, unbounded};

struct Worker {
    _id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(id) = self.thread.take() {
            id.join().unwrap();
        }
    }
}

struct Job(Box<dyn FnOnce() + Send + 'static>);

#[derive(Default)]
struct ThreadInnerPool {
    jobcount: Arc<Mutex<usize>>,
    cvar: Condvar,
}
impl ThreadInnerPool {
    fn start_job(&self) {
        let mut joblock = self.jobcount.lock().unwrap();

        *joblock += 1;

        println!("Job Started! current Jobcount:{:?}", *joblock);
    }
    fn finish_job(&self) {
        let mut joblock = self.jobcount.lock().unwrap();

        *joblock -= 1;

        println!("Job Completed! current Jobcount:{:?}", *joblock);
        if *joblock == 0 {
            println!("No Job Left! current Jobcount:{:?}", *joblock);
            self.cvar.notify_all();
        }
    }
    fn wait_empty(&self) {
        let mut joblock = self.jobcount.lock().unwrap();
        while *joblock > 0 {
            println!("Waiting!");
            joblock = self.cvar.wait(joblock).unwrap();
        }
    }
}

pub struct ThreadPool {
    _workers: Vec<Worker>,
    inner_pool: Arc<ThreadInnerPool>,

    sender: Option<Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let pool = Arc::new(ThreadInnerPool::default());
        let (s, r) = unbounded();
        let mut result = Vec::new();
        for id in 0..size {
            let pool = Arc::clone(&pool);
            let r: Receiver<Job> = r.clone();
            let thread = thread::spawn(move || {
                while let Ok(job) = r.recv() {
                    job.0();
                    pool.finish_job();
                }
            });
            result.push(Worker {
                _id: id,
                thread: Some(thread),
            });
        }
        Self {
            _workers: result,
            inner_pool: pool,
            sender: Some(s),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Job(Box::new(f));
        if let Some(sender) = &self.sender {
            sender
                .send(job)
                .expect("Failed to send job to worker thread");
            self.inner_pool.start_job();
        }
    }
    pub fn join(&self) {
        self.inner_pool.wait_empty();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender = None;
        for worker in &mut self._workers {
            if let Some(handle) = worker.thread.take() {
                handle.join().expect("Worker Thread Panicked");
            }
        }
    }
}

fn fib(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fib(n - 1) + fib(n - 2),
    }
}

pub fn th() {
    let a = ThreadPool::new(30);
    let vec = Arc::new(vec![50, 60, 70, 80, 90]);
    let result: Arc<Mutex<Vec<(usize, usize)>>> = Arc::new(Mutex::new(Vec::new()));
    let cache = SharedFibCache::new();
    for &i in vec.iter() {
        let cache = cache.clone();
        let result = Arc::clone(&result);

        a.execute(move || {
            let res = cache.get(i);
            let mut reslock = result.lock().unwrap();
            reslock.push((i, res));
        });
    }
    a.join();
    for (input, result) in result.lock().unwrap().iter() {
        println!("Fib of {input} is {result}");
    }
    println!("{:?}", cache.print());
}

struct FibCache {
    data: Vec<usize>,
}

impl FibCache {
    pub fn new() -> FibCache {
        FibCache { data: vec![0, 1] }
    }
    pub fn compute_up_to(&mut self, n: usize) -> usize {
        let current_len = self.data.len();
        if n < current_len {
            self.data[n];
        }
        for i in current_len..=n {
            let r = self.data[i - 1] + self.data[i - 2];
            self.data.push(r);
        }
        self.data[n]
    }
}
#[derive(Clone)]
pub struct SharedFibCache {
    inner: Arc<Mutex<FibCache>>,
}

impl SharedFibCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(FibCache::new())),
        }
    }
    pub fn get(&self, n: usize) -> usize {
        let mut cache = self.inner.lock().unwrap();
        cache.compute_up_to(n)
    }
    pub fn print(&self) {
        println!("{:?}", self.inner.lock().unwrap().data)
    }
}
