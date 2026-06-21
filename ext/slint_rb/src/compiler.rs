use magnus::{RArray, Ruby};
use slint_interpreter::ComponentHandle;
use std::collections::HashMap;
use std::path::PathBuf;
use std::cell::RefCell;
use std::thread::{self, ThreadId};

#[magnus::wrap(class = "Slint::Compiler")]
pub struct Compiler {
    compiler: SendableWrapper<slint_interpreter::Compiler>
}

#[magnus::wrap(class = "Slint::CompilationResult")]
pub struct CompilationResult {
    result: SendableWrapper<slint_interpreter::CompilationResult>
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            compiler: SendableWrapper::new(Default::default())
        }
    }

    pub fn build_from_path(&self, path: PathBuf) -> CompilationResult {
        self.compiler.with(|inner| {
            let future = inner.build_from_path(path);
            let result = spin_on::spin_on(future);
            CompilationResult { result: SendableWrapper::new(result) }
        })
    }

    pub fn build_from_source(&self, source_code: String, path: PathBuf) -> CompilationResult {
        self.compiler.with(|inner| {
            let future = inner.build_from_source(source_code, path);
            let result = spin_on::spin_on(future);

            CompilationResult { result: SendableWrapper::new(result) }
        })
    }

    pub fn include_paths(&self) -> Vec<String> {
        self.compiler.with(|inner| {
            inner
                .include_paths()
                .iter()
                .map(|p| p.to_str().unwrap_or_default().to_string())
                .collect()
        })
    }

    pub fn set_include_paths(&self, include_paths: Vec<PathBuf>) {
        self.compiler.with_mut(|inner| inner.set_include_paths(include_paths))
    }

    pub fn library_paths(&self) -> HashMap<String, PathBuf> {
        self.compiler.with(|inner| {
            let mut paths = HashMap::new();

            for (key, path) in inner.library_paths() {
                paths.insert(key.clone(), path.clone());
            }

            paths
        })
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
            ruby.ary_from_iter(inner.diagnostics().map(|d| Diagnostic {
                diagnostic: SendableWrapper::new(d)
            }))
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
            ruby.ary_from_iter(inner.components().map(|c| ComponentDefinition {
                definition: SendableWrapper::new(c)
            }))
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
                _ => ruby.sym_new("UNKNOWN")    
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
        self.diagnostic.with(|inner| inner.source_file().map(|sf| sf.to_owned()))
    }
}

#[magnus::wrap(class = "Slint::ComponentDefinition")]
pub struct ComponentDefinition {
    definition: SendableWrapper<slint_interpreter::ComponentDefinition>
}

impl ComponentDefinition {
    pub fn render(&self) {
        self.definition.with(|inner| {
            let instance = inner.create().unwrap();
            instance.run().unwrap();
        })
    }
}

struct SendableWrapper<T> {
    value: RefCell<T>,
    thread_id: ThreadId
}

impl<T> SendableWrapper<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            thread_id: thread::current().id()
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.value.borrow();
        f(&guard)
    }

    pub fn with_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.value.borrow_mut();
        f(&mut guard)
    }
}

unsafe impl<T> Send for SendableWrapper<T> {}