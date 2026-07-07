use magnus::{function, method, prelude::*, Error, Ruby};

mod compiler;
mod sendable_wrapper;

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

    let compilation_result_class = module.define_class("CompilationResult", ruby.class_object())?;
    compilation_result_class.define_method("valid?", method!(compiler::CompilationResult::valid, 0))?;
    compilation_result_class.define_method("diagnostics", method!(compiler::CompilationResult::diagnostics, 0))?;
    compilation_result_class.define_method("component_names", method!(compiler::CompilationResult::component_names, 0))?;
    compilation_result_class.define_method("components", method!(compiler::CompilationResult::components, 0))?;

    let diagnostic_class = module.define_class("Diagnostic", ruby.class_object())?;
    diagnostic_class.define_method("message", method!(compiler::Diagnostic::message, 0))?;
    diagnostic_class.define_method("line_column", method!(compiler::Diagnostic::line_column, 0))?;
    diagnostic_class.define_method("level", method!(compiler::Diagnostic::level, 0))?;
    diagnostic_class.define_method("source_file", method!(compiler::Diagnostic::source_file, 0))?;

    let component_definition_class =
        module.define_class("ComponentDefinition", ruby.class_object())?;

    component_definition_class.define_method("create", method!(compiler::ComponentDefinition::create, 0))?;
    component_definition_class.define_method("name", method!(compiler::ComponentDefinition::name, 0))?;
    component_definition_class.define_method("callbacks", method!(compiler::ComponentDefinition::callbacks, 0))?;
    component_definition_class.define_method("functions", method!(compiler::ComponentDefinition::functions, 0))?;
    component_definition_class.define_method("properties", method!(compiler::ComponentDefinition::properties, 0))?;
    component_definition_class.define_method("globals", method!(compiler::ComponentDefinition::globals, 0))?;
    component_definition_class.define_method("global_properties", method!(compiler::ComponentDefinition::global_properties, 1))?;
    component_definition_class.define_method("global_callbacks", method!(compiler::ComponentDefinition::global_callbacks, 1))?;
    component_definition_class.define_method("global_functions", method!(compiler::ComponentDefinition::global_functions, 1))?;

    let component_instance_class = module.define_class("ComponentInstance", ruby.class_object())?;
    component_instance_class.define_method("definition", method!(compiler::ComponentInstance::definition, 0))?;
    component_instance_class.define_method("get_property", method!(compiler::ComponentInstance::get_property, 1))?;
    component_instance_class.define_method("set_property", method!(compiler::ComponentInstance::set_property, 2))?;
    component_instance_class.define_method("get_global_property", method!(compiler::ComponentInstance::get_global_property, 2))?;
    component_instance_class.define_method("set_global_property", method!(compiler::ComponentInstance::set_global_property, 3))?;
    component_instance_class.define_method("render", method!(compiler::ComponentInstance::render, 0))?;

    let brush_class = module.define_class("Brush", ruby.class_object())?;
    brush_class.define_method("transparent?", method!(compiler::Brush::is_transparent, 0))?;
    Ok(())
}
