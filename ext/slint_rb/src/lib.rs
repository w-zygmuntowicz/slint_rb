use magnus::{prelude::*, function, method, Error, Ruby};

mod compiler_wrapper;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Slint")?;

    let wrapper = module.define_class("Compiler", ruby.class_object())?;
    wrapper.define_singleton_method("new", function!(compiler_wrapper::CompilerWrapper::new, 0))?;
    wrapper.define_method("build_from_path", method!(compiler_wrapper::CompilerWrapper::build_from_path, 1))?;
    Ok(())
}
