#![forbid(unsafe_code)]

use crossbeam::channel::{self, Sender};

use std::{
    borrow::Borrow,
    mem::swap,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{Arc, Mutex},
    thread,
};

////////////////////////////////////////////////////////////////////////////////

pub struct ResultWrapper<T> {
    flag: bool,
    result: Result<T, JoinError>,
}

impl<T> ResultWrapper<T> {
    pub fn take(&mut self) -> Result<T, JoinError> {
        self.flag = false;
        let mut ans = Err(JoinError {});
        swap(&mut ans, &mut self.result);
        ans
    }
}

pub struct ThreadPool {
    inner_handles: Vec<thread::JoinHandle<()>>,
    sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
    status: Arc<Mutex<bool>>,
}

impl ThreadPool {
    pub fn new(thread_count: usize, queue_size: usize) -> Self {
        let (sender, receiver) = channel::bounded::<Box<dyn FnOnce() + Send + 'static>>(queue_size);
        let status = Arc::new(Mutex::new(true));
        let inner_handles: Vec<thread::JoinHandle<()>> = (0..thread_count)
            .map(|_| {
                let c_receiver = receiver.clone();
                let c_status = status.clone();
                thread::spawn(move || loop {
                    if !*c_status.lock().unwrap() {
                        return;
                    }
                    let Ok(task) = c_receiver.try_recv() else {
                        continue;
                    };
                    (task)();
                })
            })
            .collect();
        Self {
            sender,
            inner_handles,
            status,
        }
    }

    pub fn spawn<Task, Answer>(&self, task: Task) -> JoinHandle<Answer>
    where
        Task: FnOnce() -> Answer,
        Task: Send + 'static,
        Answer: Send + 'static,
    {
        let control = Arc::new(Mutex::new(ResultWrapper::<Answer> {
            flag: false,
            result: Err(JoinError {}),
        }));
        let c_control = control.clone();
        let query = Box::new(move || {
            //println!("Starting query");
            let query_result: Result<Answer, JoinError> = match catch_unwind(AssertUnwindSafe(task))
            {
                Ok(value) => Ok(value),
                _ => Err(JoinError {}),
            };
            let mtx: &Mutex<ResultWrapper<Answer>> = c_control.borrow();
            let mut data = mtx.lock().unwrap();
            data.flag = true;
            data.result = query_result;
        });
        self.sender.send(query).unwrap();
        JoinHandle::<Answer> { inner: control }
    }

    pub fn shutdown(self) {
        while !self.sender.is_empty() {}
        *self.status.lock().unwrap() = false;
        for handle in self.inner_handles.into_iter() {
            handle.join().unwrap();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct JoinHandle<T> {
    inner: Arc<Mutex<ResultWrapper<T>>>,
}

#[derive(Debug)]
pub struct JoinError {}

impl<T> JoinHandle<T> {
    pub fn join(self) -> Result<T, JoinError> {
        loop {
            let mtx: &Mutex<ResultWrapper<T>> = self.inner.borrow();
            let mut data = mtx.lock().unwrap();
            if data.flag {
                return data.take();
            }
        }
    }
}
