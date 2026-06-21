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

    pub fn build_from_path(&self, path: PathBuf) -> CompilationResult {
        self.compiler.with(|inner| {
            let future = inner.build_from_path(path);
            let result = spin_on::spin_on(future);
            result.into()
        })
    }

    pub fn build_from_source(&self, source_code: String, path: PathBuf) -> CompilationResult {
        self.compiler.with(|inner| {
            let future = inner.build_from_source(source_code, path);
            let result = spin_on::spin_on(future);
            result.into()
        })
    }

    pub fn include_paths(&self) -> Vec<PathBuf> {
        self.compiler.with(|inner| { inner.include_paths().to_vec() })
    }

    pub fn set_include_paths(&self, include_paths: Vec<PathBuf>) {
        self.compiler.with_mut(|inner| inner.set_include_paths(include_paths))
    }

    pub fn library_paths(&self) -> HashMap<String, PathBuf> {
        self.compiler.with(|inner| { inner.library_paths().clone() })
    }

    pub fn set_library_paths(&self, library_paths: HashMap<String, PathBuf>) {
        self.compiler.with_mut(|inner| inner.set_library_paths(library_paths))
    }

    pub fn style(&self) -> Option<String> {
        self.compiler.with(|inner| inner.style().cloned())
    }

    pub fn set_style(&self, style: String) {
        self.compiler.with_mut(|inner| inner.set_style(style))
    }
}

impl CompilationResult {
    pub fn valid(&self) -> bool {
        self.result.with(|inner| !inner.has_errors())
    }

    pub fn diagnostics(ruby: &Ruby, rb_self: &Self) -> RArray {
        rb_self.result.with(|inner| {
            ruby.ary_from_iter(inner.diagnostics().map(Diagnostic::from))
        })
    }

    pub fn component_names(&self) -> Vec<String> {
        self.result.with(|inner| {
            inner
                .component_names()
                .map(|name| name.to_string())
                .collect()
        })
    }

    pub fn components(ruby: &Ruby, rb_self: &Self) -> RArray {
        rb_self.result.with(|inner| {
            ruby.ary_from_iter(inner.components().map(ComponentDefinition::from))
        })
    }
}

#[magnus::wrap(class = "Slint::Diagnostic")]
pub struct Diagnostic {
    diagnostic: SendableWrapper<slint_interpreter::Diagnostic>
}

impl Diagnostic {
    pub fn level(ruby: &Ruby, rb_self: &Self) -> magnus::StaticSymbol {
        rb_self.diagnostic.with(|inner| {
            match inner.level() {
                slint_interpreter::DiagnosticLevel::Error => ruby.sym_new("error"),
                slint_interpreter::DiagnosticLevel::Warning => ruby.sym_new("warning"),
                _ => ruby.sym_new("unknown")    
            }
        })
    }

    pub fn message(&self) -> String {
        self.diagnostic.with(|inner| inner.message().to_string())
    }

    pub fn line_column(&self) -> (usize, usize) {
        self.diagnostic.with(|inner| inner.line_column())
    }

    pub fn source_file(&self) -> Option<PathBuf> {
        self.diagnostic.with(|inner| inner.source_file().map(Path::to_path_buf))
    }
}

#[magnus::wrap(class = "Slint::ComponentDefinition")]
pub struct ComponentDefinition {
    definition: SendableWrapper<slint_interpreter::ComponentDefinition>
}

impl ComponentDefinition {
    pub fn create(ruby: &Ruby, rb_self: &Self) -> Result<ComponentInstance, magnus::Error> {
        rb_self.definition.with(|inner| {
            match inner.create() {
                Ok(instance) => Ok(instance.into()),
                Err(e) => Err(magnus::Error::new(ruby.exception_standard_error(), e.to_string())),
            }
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
    pub fn render(&self) {
        self.instance.with(|inner| inner.run().unwrap())
    }
}
