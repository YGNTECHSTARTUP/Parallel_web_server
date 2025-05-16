use std::{sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread::{self, JoinHandle}};
type Job = Box<dyn FnOnce()+ 'static +Send >;
pub struct ThreadPool{
    sender:Sender<Job>,
    workers:Vec<Workers>
    
}

impl ThreadPool {
    pub fn new(size: usize)->ThreadPool {
        let (tx,rx) = mpsc::channel();
        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
        workers.push(Workers::new(id,Arc::clone(&rx)));
        }
        ThreadPool{
            workers,
            sender:tx
        }
    }
    pub fn execute<F>(&self,f:F)where F:'static + Send + FnOnce() {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
        
    }
}
 struct Workers {
    id:usize,
    threads:JoinHandle<()>
}

impl Workers {
    pub fn new(id:usize,rx:Arc<Mutex<Receiver<Job>>>)->Workers {
        let threads = thread::spawn(move ||{
            loop{
            let job = rx.lock().unwrap().recv().unwrap();
            println!("Worker {id} got Job");
            job();
            }
        });
        Workers { id, threads  }
    }
}

