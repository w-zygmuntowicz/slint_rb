use magnus::prelude::*;
use magnus::Ruby;

pub type RbResult<T> = Result<T, magnus::Error>;

pub struct SlintError {}

impl SlintError {
    pub fn new_err(msg: String) -> magnus::Error {
        let class = Ruby::get()
            .unwrap()
            .class_object()
            .const_get::<_, magnus::RModule>("Slint")
            .unwrap()
            .const_get("Error")
            .unwrap();

        magnus::Error::new(class, msg)
    }
}
