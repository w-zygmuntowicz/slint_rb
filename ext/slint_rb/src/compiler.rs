use magnus::{RArray, Ruby};
use slint_interpreter::ComponentHandle;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::sendable_wrapper::SendableWrapper;

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

    pub fn create(ruby: &Ruby, rb_self: &Self) -> Result<ComponentInstance, magnus::Error> {
        rb_self.with(|inner| {
            match inner.create() {
                Ok(instance) => Ok(instance.into()),
                Err(e) => Err(magnus::Error::new(ruby.exception_standard_error(), e.to_string())),
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

    pub fn properties(ruby: &Ruby, rb_self: &Self) -> Result<magnus::RHash, magnus::Error> {
        rb_self.with(|inner| {
            Self::properties_to_hash(ruby, inner.properties())
        })
    }

    fn properties_to_hash(
        ruby: &Ruby,
        props: impl IntoIterator<Item = (String, slint_interpreter::ValueType)>,
    ) -> Result<magnus::RHash, magnus::Error> {
        props
            .into_iter()
            .map(|(name, val_type)| (name, Self::value_type_to_symbol(ruby, &val_type)))
            .try_fold(ruby.hash_new(), |acc, (name, val_type)| {
                acc.aset(name, val_type)?;
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

    pub fn global_properties(ruby: &Ruby, rb_self: &Self, global_name: String) -> Result<Option<magnus::RHash>, magnus::Error> {
        rb_self.with(|inner| {
            inner.global_properties(&global_name)
                .map(|props| Self::properties_to_hash(ruby, props))
                .transpose()
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

    pub fn render(&self) {
        self.with(|inner| inner.run().unwrap())
    }
}
