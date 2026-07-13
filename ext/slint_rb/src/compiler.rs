use magnus::{RModule, prelude::*};
use magnus::{RArray, Ruby, function, method};
use slint_interpreter::{ComponentHandle};
use slint_interpreter::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::errors::{RbResult, SlintError};
use crate::sendable_wrapper::SendableWrapper;
use crate::brush::{Brush, Color};

#[magnus::wrap(class = "Slint::Compiler")]
pub struct Compiler {
    compiler: SendableWrapper<slint_interpreter::Compiler>
}

#[magnus::wrap(class = "Slint::CompilationResult")]
pub struct CompilationResult {
    result: SendableWrapper<slint_interpreter::CompilationResult>
}

impl From<slint_interpreter::CompilationResult> for CompilationResult {
    fn from(result: slint_interpreter::CompilationResult) -> Self {
        Self {
            result: SendableWrapper::new(result)
        }
    }
}

impl From<slint_interpreter::Diagnostic> for Diagnostic {
    fn from(diagnostic: slint_interpreter::Diagnostic) -> Self {
        Self {
            diagnostic: SendableWrapper::new(diagnostic)
        }
    }
}

impl From<slint_interpreter::ComponentDefinition> for ComponentDefinition {
    fn from(component_definition: slint_interpreter::ComponentDefinition) -> Self {
        Self {
            definition: SendableWrapper::new(component_definition)
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self { compiler: SendableWrapper::new(Default::default()) }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    fn with<R>(&self, f: impl FnOnce(&slint_interpreter::Compiler) -> R) -> R {
        self.compiler.with(f)
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut slint_interpreter::Compiler) -> R) -> R {
        self.compiler.with_mut(f)
    }

    pub fn build_from_path(&self, path: PathBuf) -> CompilationResult {
        self.with(|inner| {
            let future = inner.build_from_path(path);
            let result = spin_on::spin_on(future);
            result.into()
        })
    }

    pub fn build_from_source(&self, source_code: String, path: PathBuf) -> CompilationResult {
        self.with(|inner| {
            let future = inner.build_from_source(source_code, path);
            let result = spin_on::spin_on(future);
            result.into()
        })
    }

    pub fn include_paths(&self) -> Vec<PathBuf> {
        self.with(|inner| inner.include_paths().to_vec())
    }

    pub fn set_include_paths(&self, include_paths: Vec<PathBuf>) {
        self.with_mut(|inner| inner.set_include_paths(include_paths))
    }

    pub fn library_paths(&self) -> HashMap<String, PathBuf> {
        self.with(|inner| inner.library_paths().clone())
    }

    pub fn set_library_paths(&self, library_paths: HashMap<String, PathBuf>) {
        self.with_mut(|inner| inner.set_library_paths(library_paths))
    }

    pub fn style(&self) -> Option<String> {
        self.with(|inner| inner.style().cloned())
    }

    pub fn set_style(&self, style: String) {
        self.with_mut(|inner| inner.set_style(style))
    }
}

impl CompilationResult {
    fn with<R>(&self, f: impl FnOnce(&slint_interpreter::CompilationResult) -> R) -> R {
        self.result.with(f)
    }

    pub fn valid(&self) -> bool {
        self.with(|inner| !inner.has_errors())
    }

    pub fn diagnostics(ruby: &Ruby, rb_self: &Self) -> RArray {
        rb_self.with(|inner| {
            ruby.ary_from_iter(inner.diagnostics().map(Diagnostic::from))
        })
    }

    pub fn component_names(&self) -> Vec<String> {
        self.with(|inner| {
            inner
                .component_names()
                .map(|name| name.to_string())
                .collect()
        })
    }

    pub fn components(ruby: &Ruby, rb_self: &Self) -> RArray {
        rb_self.with(|inner| {
            ruby.ary_from_iter(inner.components().map(ComponentDefinition::from))
        })
    }
}

#[magnus::wrap(class = "Slint::Diagnostic")]
pub struct Diagnostic {
    diagnostic: SendableWrapper<slint_interpreter::Diagnostic>
}

impl Diagnostic {
    fn with<R>(&self, f: impl FnOnce(&slint_interpreter::Diagnostic) -> R) -> R {
        self.diagnostic.with(f)
    }

    pub fn level(ruby: &Ruby, rb_self: &Self) -> magnus::StaticSymbol {
        rb_self.with(|inner| {
            match inner.level() {
                slint_interpreter::DiagnosticLevel::Error => ruby.sym_new("error"),
                slint_interpreter::DiagnosticLevel::Warning => ruby.sym_new("warning"),
                _ => ruby.sym_new("unknown")    
            }
        })
    }

    pub fn message(&self) -> String {
        self.with(|inner| inner.message().to_string())
    }

    pub fn line_column(&self) -> (usize, usize) {
        self.with(|inner| inner.line_column())
    }

    pub fn source_file(&self) -> Option<PathBuf> {
        self.with(|inner| inner.source_file().map(Path::to_path_buf))
    }
}

#[magnus::wrap(class = "Slint::ComponentDefinition")]
pub struct ComponentDefinition {
    definition: SendableWrapper<slint_interpreter::ComponentDefinition>
}

impl ComponentDefinition {
    fn with<R>(&self, f: impl FnOnce(&slint_interpreter::ComponentDefinition) -> R) -> R {
        self.definition.with(f)
    }

    pub fn create(&self) -> RbResult<ComponentInstance> {
        self.with(|inner| {
            match inner.create() {
                Ok(instance) => Ok(instance.into()),
                Err(e) => Err(SlintError::new_err(e.to_string())),
            }
        })
    }

    pub fn name(&self) -> String {
        self.with(|inner| inner.name().to_string())
    }

    pub fn callbacks(&self) -> Vec<String> {
        self.with(|inner| inner.callbacks().collect())
    }

    pub fn functions(&self) -> Vec<String> {
        self.with(|inner| inner.functions().collect())
    }

    pub fn properties(ruby: &Ruby, rb_self: &Self) -> RbResult<magnus::RHash> {
        rb_self.with(|inner| {
            Self::properties_to_hash(ruby, inner.properties())
        })
    }

    fn properties_to_hash(
        ruby: &Ruby,
        props: impl IntoIterator<Item = (String, slint_interpreter::ValueType)>,
    ) -> RbResult<magnus::RHash> {
        props
            .into_iter()
            .map(|(name, val_type)| (name, Self::value_type_to_symbol(ruby, &val_type)))
            .try_fold(ruby.hash_new(), |acc, (name, val_type)| {
                acc.aset(name, val_type).map_err(|e| SlintError::new_err(e.to_string()))?;
                Ok(acc)
            })
    }

    fn value_type_to_symbol(ruby: &Ruby, value_type: &slint_interpreter::ValueType) -> magnus::StaticSymbol {
        match value_type {
            slint_interpreter::ValueType::Void => ruby.sym_new("void"),
            slint_interpreter::ValueType::Number => ruby.sym_new("number"),
            slint_interpreter::ValueType::String => ruby.sym_new("string"),
            slint_interpreter::ValueType::Bool => ruby.sym_new("bool"),
            slint_interpreter::ValueType::Struct => ruby.sym_new("struct"),
            slint_interpreter::ValueType::Brush => ruby.sym_new("brush"),
            slint_interpreter::ValueType::Image => ruby.sym_new("image"),
            slint_interpreter::ValueType::Model => ruby.sym_new("model"),
            _ => ruby.sym_new("unknown")
        }
    }

    pub fn globals(&self) -> Vec<String> {
        self.with(|inner| inner.globals().collect())
    }

    pub fn global_properties(ruby: &Ruby, rb_self: &Self, global_name: String) -> RbResult<Option<magnus::RHash>> {
        rb_self.with(|inner| {
            inner.global_properties(&global_name)
                .map(|props| Self::properties_to_hash(ruby, props))
                .transpose()
                .map_err(|e| SlintError::new_err(e.to_string()))
        })
    }

    pub fn global_callbacks(&self, global_name: String) -> Option<Vec<String>> {
        self.with(|inner| {
            inner.global_callbacks(&global_name)
                .map(Iterator::collect) 
        })
    }

    pub fn global_functions(&self, global_name: String) -> Option<Vec<String>> {
        self.with(|inner| {
            inner.global_functions(&global_name)
                .map(Iterator::collect) 
        })
    }
}

impl From<slint_interpreter::ComponentInstance> for ComponentInstance {
    fn from(instance: slint_interpreter::ComponentInstance) -> Self {
        Self {
            instance: SendableWrapper::new(instance)
        }
    }
}

#[magnus::wrap(class = "Slint::ComponentInstance")]
pub struct ComponentInstance {
    instance: SendableWrapper<slint_interpreter::ComponentInstance>
}

impl ComponentInstance {
    fn with<R>(&self, f: impl FnOnce(&slint_interpreter::ComponentInstance) -> R) -> R {
        self.instance.with(f)
    }

    pub fn definition(&self) -> ComponentDefinition {
        self.with(|inner| inner.definition().into())
    }

    pub fn get_property(ruby: &Ruby, rb_self: &Self, property_name: String) -> RbResult<magnus::Value> {
        rb_self.with(|inner| {
            inner
                .get_property(&property_name)
                .map_err(|e| SlintError::new_err(e.to_string()))
                .map(|property| Self::try_slint_value_into_ruby_value(ruby, &property))?
        })
    }

    pub fn get_global_property(
        ruby: &Ruby,
        rb_self: &Self,
        global_name: String,
        property_name: String,
    ) -> RbResult<magnus::Value> {
        rb_self.with(|inner| {
            inner
                .get_global_property(&global_name, &property_name)
                .map_err(|e| SlintError::new_err(e.to_string()))
                .map(|property| Self::try_slint_value_into_ruby_value(ruby, &property))?
        })
    }

    pub fn set_property(ruby: &Ruby, rb_self: &Self, property_name: String, new_value: magnus::Value) -> RbResult<magnus::Value> {
        rb_self.with(|inner| {
            let old_value = inner
                .get_property(&property_name)
                .map_err(|e| SlintError::new_err(e.to_string()))?;
            let converted_value = Self::try_slint_value_from_ruby_value(ruby, old_value, new_value)?;
            inner
                .set_property(&property_name, converted_value)
                .map_err(|e| SlintError::new_err(e.to_string()))?;
            Ok(new_value)
        })
    }

    pub fn set_global_property(
        ruby: &Ruby,
        rb_self: &Self,
        global_name: String,
        property_name: String,
        new_value: magnus::Value,
    ) -> RbResult<magnus::Value> {
        rb_self.with(|inner| {
            let old_value = inner
                .get_global_property(&global_name, &property_name)
                .map_err(|e| SlintError::new_err(e.to_string()))?;
            let converted_value = Self::try_slint_value_from_ruby_value(ruby, old_value, new_value)?;
            inner
                .set_global_property(&global_name, &property_name, converted_value)
                .map_err(|e| SlintError::new_err(e.to_string()))?;
            Ok(new_value)
        })
    }

    fn try_slint_value_from_ruby_value(_ruby: &Ruby, old_value: Value, value: magnus::Value) -> RbResult<Value> {
        match old_value {
            Value::Number(_) => {
                let val = f64::try_convert(value)?;
                Ok(Value::Number(val))
            }
            Value::String(_) => {
                let val = String::try_convert(value)?;
                Ok(Value::String(val.into()))
            }
            Value::Bool(_) => {
                let val = bool::try_convert(value)?;
                Ok(Value::Bool(val))
            }
            Value::Brush(slint_interpreter::Brush::SolidColor(_)) => {
                if let Ok(brush) = <&Brush>::try_convert(value) {
                    Ok(Value::Brush(brush.into()))
                } else if let Ok(color) = <&Color>::try_convert(value) {
                    Ok(Value::Brush(slint_interpreter::Brush::SolidColor(color.into())))
                } else {
                    <&Brush>::try_convert(value)?;
                    unreachable!()
                }
            }
            _ => Err(SlintError::new_err(format!(
                "Setting property of type {:?} is not supported yet",
                old_value
            ))),
        }
    }

    fn try_slint_value_into_ruby_value(ruby: &Ruby, property: &Value) -> RbResult<magnus::Value> {
        match property {
            Value::Number(number) => Ok(ruby.into_value(*number)),
            Value::String(text) => Ok(ruby.into_value(text.as_str())),
            Value::Bool(value) => Ok(ruby.into_value(*value)),
            Value::Brush(brush) => Ok(ruby.into_value(Brush::from(brush.clone()))),
            _ => Err(SlintError::new_err("Property mapping to ruby not implemented yet for this type".to_string()))

            // /// There is nothing in this value. That's the default.
            // /// For example, a function that does not return a result would return a Value::Void
            // #[default]
            // Void = 0,
            // /// A model (that includes array in .slint)
            // Model(ModelRc<Value>) = 5,
            // /// An object
            // Struct(Struct) = 6,
            // /// Correspond to `brush` or `color` type in .slint.  For color, this is then a [`Brush::SolidColor`]
            // Brush(Brush) = 7,
            // #[doc(hidden)]
            // /// The elements of a path
            // PathData(PathData) = 8,
            // #[doc(hidden)]
            // /// An easing curve
            // EasingCurve(i_slint_core::animations::EasingCurve) = 9,
            // #[doc(hidden)]
            // /// An enumeration, like `TextHorizontalAlignment::align_center`, represented by `("TextHorizontalAlignment", "align_center")`.
            // /// FIXME: consider representing that with a number?
            // EnumerationValue(String, String) = 10,
            // #[doc(hidden)]
            // LayoutCache(SharedVector<f32>) = 11,
            // #[doc(hidden)]
            // /// Correspond to the `component-factory` type in .slint
            // ComponentFactory(ComponentFactory) = 12,
            // #[doc(hidden)] // make visible when we make StyledText public
            // /// Correspond to the `styled-text` type in .slint
            // StyledText(StyledText) = 13,
            // #[doc(hidden)]
            // ArrayOfU16(SharedVector<u16>) = 14,
            // /// Correspond to the `keys` type in .slint
            // Keys(Keys) = 15,
            // /// Correspond to the `data-transfer` type in .slint
            // DataTransfer(DataTransfer) = 16,
        }
    }

    pub fn render(&self) {
        self.with(|inner| inner.run().unwrap())
    }
}

pub fn init(ruby: &Ruby, slint_module: &RModule) -> RbResult<()> {
    let compiler_class = slint_module.define_class("Compiler", ruby.class_object())?;
    compiler_class.define_singleton_method("new", function!(Compiler::new, 0))?;
    compiler_class.define_method("build_from_path", method!(Compiler::build_from_path, 1))?;
    compiler_class.define_method("build_from_source", method!(Compiler::build_from_source, 2))?;
    compiler_class.define_method("include_paths", method!(Compiler::include_paths, 0))?;
    compiler_class.define_method("include_paths=", method!(Compiler::set_include_paths, 1))?;
    compiler_class.define_method("library_paths", method!(Compiler::library_paths, 0))?;
    compiler_class.define_method("library_paths=", method!(Compiler::set_library_paths, 1))?;
    compiler_class.define_method("style", method!(Compiler::style, 0))?;
    compiler_class.define_method("style=", method!(Compiler::set_style, 1))?;

    let compilation_result_class = slint_module.define_class("CompilationResult", ruby.class_object())?;
    compilation_result_class.define_method("valid?", method!(CompilationResult::valid, 0))?;
    compilation_result_class.define_method("diagnostics", method!(CompilationResult::diagnostics, 0))?;
    compilation_result_class.define_method("component_names", method!(CompilationResult::component_names, 0))?;
    compilation_result_class.define_method("components", method!(CompilationResult::components, 0))?;

    let diagnostic_class = slint_module.define_class("Diagnostic", ruby.class_object())?;
    diagnostic_class.define_method("message", method!(Diagnostic::message, 0))?;
    diagnostic_class.define_method("line_column", method!(Diagnostic::line_column, 0))?;
    diagnostic_class.define_method("level", method!(Diagnostic::level, 0))?;
    diagnostic_class.define_method("source_file", method!(Diagnostic::source_file, 0))?;

    let component_definition_class =
        slint_module.define_class("ComponentDefinition", ruby.class_object())?;

    component_definition_class.define_method("create", method!(ComponentDefinition::create, 0))?;
    component_definition_class.define_method("name", method!(ComponentDefinition::name, 0))?;
    component_definition_class.define_method("callbacks", method!(ComponentDefinition::callbacks, 0))?;
    component_definition_class.define_method("functions", method!(ComponentDefinition::functions, 0))?;
    component_definition_class.define_method("properties", method!(ComponentDefinition::properties, 0))?;
    component_definition_class.define_method("globals", method!(ComponentDefinition::globals, 0))?;
    component_definition_class.define_method("global_properties", method!(ComponentDefinition::global_properties, 1))?;
    component_definition_class.define_method("global_callbacks", method!(ComponentDefinition::global_callbacks, 1))?;
    component_definition_class.define_method("global_functions", method!(ComponentDefinition::global_functions, 1))?;

    let component_instance_class = slint_module.define_class("ComponentInstance", ruby.class_object())?;
    component_instance_class.define_method("definition", method!(ComponentInstance::definition, 0))?;
    component_instance_class.define_method("get_property", method!(ComponentInstance::get_property, 1))?;
    component_instance_class.define_method("set_property", method!(ComponentInstance::set_property, 2))?;
    component_instance_class.define_method("get_global_property", method!(ComponentInstance::get_global_property, 2))?;
    component_instance_class.define_method("set_global_property", method!(ComponentInstance::set_global_property, 3))?;
    component_instance_class.define_method("render", method!(ComponentInstance::render, 0))?;

    Ok(())
}