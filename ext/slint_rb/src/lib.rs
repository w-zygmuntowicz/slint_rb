use magnus::{prelude::*, function, method, Error, Ruby};

mod compiler;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Slint")?;

    let wrapper = module.define_class("Compiler", ruby.class_object())?;
    wrapper.define_singleton_method("new", function!(compiler::Compiler::new, 0))?;
    wrapper.define_method("build_from_path", method!(compiler::Compiler::build_from_path, 1))?;

    let compilation_result = module.define_class("CompilationResult", ruby.class_object())?;
    compilation_result.define_method("render", method!(compiler::CompilationResultWrapper::render, 0))?;
    Ok(())
}
