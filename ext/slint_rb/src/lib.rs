use magnus::{function, method, prelude::*, Error, Ruby};

mod compiler;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Slint")?;

    let compiler_class = module.define_class("Compiler", ruby.class_object())?;
    compiler_class.define_singleton_method("new", function!(compiler::Compiler::new, 0))?;
    compiler_class.define_method(
        "build_from_path",
        method!(compiler::Compiler::build_from_path, 1),
    )?;
    compiler_class.define_method(
        "build_from_source",
        method!(compiler::Compiler::build_from_source, 2),
    )?;
    compiler_class.define_method(
        "include_paths",
        method!(compiler::Compiler::include_paths, 0),
    )?;
    compiler_class.define_method(
        "include_paths=",
        method!(compiler::Compiler::set_include_paths, 1),
    )?;
    compiler_class.define_method(
        "library_paths",
        method!(compiler::Compiler::library_paths, 0),
    )?;
    compiler_class.define_method(
        "library_paths=",
        method!(compiler::Compiler::set_library_paths, 1),
    )?;
    compiler_class.define_method("style", method!(compiler::Compiler::style, 0))?;
    compiler_class.define_method("style=", method!(compiler::Compiler::set_style, 1))?;

    let compilation_result_class = module.define_class("CompilationResult", ruby.class_object())?;
    compilation_result_class
        .define_method("valid?", method!(compiler::CompilationResult::valid, 0))?;
    compilation_result_class.define_method(
        "diagnostics",
        method!(compiler::CompilationResult::diagnostics, 0),
    )?;
    compilation_result_class.define_method(
        "component_names",
        method!(compiler::CompilationResult::component_names, 0),
    )?;
    compilation_result_class.define_method(
        "components",
        method!(compiler::CompilationResult::components, 0),
    )?;

    let diagnostic_class = module.define_class("Diagnostic", ruby.class_object())?;
    diagnostic_class.define_method("message", method!(compiler::Diagnostic::message, 0))?;
    diagnostic_class.define_method("line_column", method!(compiler::Diagnostic::line_column, 0))?;
    diagnostic_class.define_method("level", method!(compiler::Diagnostic::level, 0))?;
    diagnostic_class.define_method("source_file", method!(compiler::Diagnostic::source_file, 0))?;

    let component_definition_class =
        module.define_class("ComponentDefinition", ruby.class_object())?;

    component_definition_class
        .define_method("render", method!(compiler::ComponentDefinition::render, 0))?;
    Ok(())
}
