use std::cell::RefCell;
use std::thread::{self, ThreadId};

pub struct SendableWrapper<T> {
    value: RefCell<T>,
    thread_id: ThreadId
}

impl<T> SendableWrapper<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            thread_id: thread::current().id()
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.value.borrow();
        f(&guard)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.value.borrow_mut();
        f(&mut guard)
    }
}

unsafe impl<T> Send for SendableWrapper<T> {}