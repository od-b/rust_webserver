#![allow(dead_code)]
#![allow(unused_imports)]

use std::{
    sync::{mpsc, Arc, Mutex, PoisonError},
    thread::{self, JoinHandle},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // spawn a dormant thread
        let handle = thread::spawn(move || loop {
            // lock mutex, unwrap LockResult
            let guard = receiver.lock().unwrap();

            // wait for a message
            let recv = guard.recv();

            // unlock mutex
            drop(guard);

            match recv {
                Ok(job) => {
                    eprintln!("Worker {id} accepting job.");
                    job();
                }
                Err(_) => {
                    eprintln!("Worker {id} disconnected; shutting down.");
                    // break out of the thread loop
                    break;
                }
            }
        });

        // return the worker containing the JoinHandle for the dormant thread
        Worker {
            id,
            handle: Some(handle),
        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, func: F)
    where
        // FnOnce --> Use FnOnce as a bound when you want to accept a parameter of
        //            function-like type and only need to *call* it once.
        // Send --> implements Send, e.g. Arc
        // static --> function lives for as long as binary
        F: FnOnce() + Send + 'static,
    {
        self.sender
            .as_ref()
            .unwrap() // unwrap sender as a ref
            .send(Box::new(func))
            .unwrap(); // box & send the job to the worker channel
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // drop the channel sender, blocking new jobs
        drop(self.sender.take());

        for worker in &mut self.workers {
            eprintln!("Shutting down worker {}", worker.id);

            // replace the handle with none, then join & unwrap
            if let Some(handle) = worker.handle.take() {
                if let Err(msg) = handle.join() {
                    eprintln!("Failed to join worker handle: {:#?}", msg);
                }
            }
        }
    }
}


// fn consume_with_relish<F>(func: F)
//     where F: FnOnce() -> String
// {
//     // `func` consumes its captured variables, so it cannot be run more
//     // than once.
//     println!("Consumed: {}", func());

//     println!("Delicious!");

//     // Attempting to invoke `func()` again will throw a `use of moved
//     // value` error for `func`.
// }
