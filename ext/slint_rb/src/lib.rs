use magnus::{prelude::*, function, method, Error, Ruby};

mod compiler;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Slint")?;

    let compiler_class = module.define_class("Compiler", ruby.class_object())?;
    compiler_class.define_singleton_method("new", function!(compiler::Compiler::new, 0))?;
    compiler_class.define_method("build_from_path", method!(compiler::Compiler::build_from_path, 1))?;
    compiler_class.define_method("build_from_source", method!(compiler::Compiler::build_from_source, 2))?;
    compiler_class.define_method("include_paths", method!(compiler::Compiler::include_paths, 0))?;
    compiler_class.define_method("include_paths=", method!(compiler::Compiler::set_include_paths, 1))?;
    compiler_class.define_method("library_paths", method!(compiler::Compiler::library_paths, 0))?;
    compiler_class.define_method("library_paths=", method!(compiler::Compiler::set_library_paths, 1))?;
    compiler_class.define_method("style", method!(compiler::Compiler::style, 0))?;
    compiler_class.define_method("style=", method!(compiler::Compiler::set_style, 1))?;

    let compilation_result = module.define_class("CompilationResult", ruby.class_object())?;
    compilation_result.define_method("render", method!(compiler::CompilationResult::render, 0))?;
    Ok(())
}
