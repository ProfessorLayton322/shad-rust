#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc, rc::Weak};

use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("channel is closed")]

pub struct SendError<T: Debug> {
    pub value: T,
}

pub struct DequeWrapper<T> {
    deque: VecDeque<T>,
    active: bool,
}

impl<T> DequeWrapper<T> {
    pub fn new() -> Self {
        Self {
            deque: VecDeque::<T>::new(),
            active: true,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn kill(&mut self) {
        self.active = false;
    }

    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }

    pub fn push_back(&mut self, value: T) {
        self.deque.push_back(value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.deque.pop_front()
    }
}

impl<T> Default for DequeWrapper<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Sender<T> {
    resource: Rc<RefCell<DequeWrapper<T>>>,
}

impl<T: Debug> Sender<T> {
    pub fn send(&self, val: T) -> Result<(), SendError<T>> {
        if self.is_closed() {
            return Err(SendError { value: val });
        }
        let mut resource_ref = self.resource.borrow_mut();
        if !resource_ref.is_active() {
            return Err(SendError { value: val });
        }
        resource_ref.push_back(val);
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        if Rc::weak_count(&self.resource) == 0 {
            return true;
        }
        let resource_ref = self.resource.borrow();
        !resource_ref.is_active()
    }

    pub fn same_channel(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.resource, &other.resource)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("channel is empty")]
    Empty,
    #[error("channel is closed")]
    Closed,
}

pub struct Receiver<T> {
    resource: Weak<RefCell<DequeWrapper<T>>>,
}

impl<T: Debug> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, ReceiveError> {
        match self.resource.upgrade() {
            None => Err(ReceiveError::Closed),
            Some(rc) => {
                let mut resource_ref = rc.borrow_mut();
                if !resource_ref.is_empty() {
                    return Ok(resource_ref.pop_front().unwrap());
                }
                if !resource_ref.is_active() {
                    return Err(ReceiveError::Closed);
                }
                Err(ReceiveError::Empty)
            }
        }
    }

    pub fn close(&mut self) {
        match self.resource.upgrade() {
            None => {}
            Some(rc) => {
                let mut resource_ref = rc.borrow_mut();
                resource_ref.kill();
            }
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        match self.resource.upgrade() {
            None => {}
            Some(rc) => {
                let mut resource_ref = rc.borrow_mut();
                resource_ref.kill();
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let rc = Rc::new(RefCell::new(DequeWrapper::new()));
    let sender = Sender::<T> {
        resource: rc.clone(),
    };
    let receiver = Receiver::<T> {
        resource: Rc::<RefCell<DequeWrapper<T>>>::downgrade(&rc),
    };
    (sender, receiver)
}
