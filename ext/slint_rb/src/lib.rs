use magnus::{Error, Ruby, function, method, prelude::*};

mod compiler;
mod brush;
mod sendable_wrapper;
mod errors;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Slint")?;

    let brush_class = module.define_class("Brush", ruby.class_object())?;
    brush_class.define_method("transparent?", method!(compiler::Brush::is_transparent, 0))?;

    compiler::init(ruby, &module)?;
    brush::init(ruby, &module)?;

    Ok(())
}
