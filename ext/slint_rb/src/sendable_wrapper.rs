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
        self.assert_thread();
        let guard = self.value.borrow();
        f(&guard)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.assert_thread();
        let mut guard = self.value.borrow_mut();
        f(&mut guard)
    }

    fn assert_thread(&self) {
        assert_eq!(
            thread::current().id(),
            self.thread_id,
            "Tried to access value from external thread"
        )
    }
}

unsafe impl<T> Send for SendableWrapper<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_from_native_thread_returns_value() {
        let wrapper = SendableWrapper::new(4);

        wrapper.with(|internal| assert_eq!(*internal, 4));
    }

    #[test]
    #[should_panic(expected = "Tried to access value from external thread")]
    fn test_with_from_external_thread_panics() {
        let wrapper = thread::spawn(|| SendableWrapper::new(4)).join().unwrap();

        wrapper.with(|internal| *internal == 4);
    }


    #[test]
    fn test_with_mut_from_native_thread_returns_value() {
        let wrapper = SendableWrapper::new(4);

        wrapper.with_mut(|internal| assert_eq!(*internal, 4));
    }

    #[test]
    #[should_panic(expected = "Tried to access value from external thread")]
    fn test_with_mut_from_external_thread_panics() {
        let wrapper = thread::spawn(|| SendableWrapper::new(4)).join().unwrap();

        wrapper.with_mut(|internal| *internal == 4);
    }
}