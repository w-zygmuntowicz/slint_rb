use magnus::{Ruby};

mod compiler;
mod brush;
mod sendable_wrapper;
mod errors;

use crate::errors::RbResult;

#[magnus::init]
fn init(ruby: &Ruby) -> RbResult<()> {
    let module = ruby.define_module("Slint")?;

    compiler::init(ruby, &module)?;
    brush::init(ruby, &module)?;

    Ok(())
}
