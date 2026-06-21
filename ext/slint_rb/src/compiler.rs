use magnus::{RArray, Ruby};
use slint_interpreter::ComponentHandle;
use std::collections::HashMap;
use std::path::PathBuf;
use std::cell::RefCell;
use std::thread::{self, ThreadId};

#[magnus::wrap(class = "Slint::Compiler")]
pub struct Compiler {
    compiler: MyWrapper<slint_interpreter::Compiler>
}

#[magnus::wrap(class = "Slint::CompilationResult")]
pub struct CompilationResult {
    result: MyWrapper<slint_interpreter::CompilationResult>
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            compiler: MyWrapper::new(Default::default())
        }
    }

    pub fn build_from_path(&self, path: PathBuf) -> CompilationResult {
        let t = self.compiler.value.borrow();
        let future = t.build_from_path(path);
        let result = spin_on::spin_on(future);
        CompilationResult { result: MyWrapper::new(result) }
    }

    pub fn build_from_source(&self, source_code: String, path: PathBuf) -> CompilationResult {
        let t = self.compiler.value.borrow();
        let future = t.build_from_source(source_code, path);
        let result = spin_on::spin_on(future);

        CompilationResult { result: MyWrapper::new(result) }
    }

    pub fn include_paths(&self) -> Vec<String> {
        let t = self.compiler.value.borrow();
        
        t
            .include_paths()
            .iter()
            .map(|p| p.to_str().unwrap_or_default().to_string())
            .collect()
    }

    pub fn set_include_paths(&self, include_paths: Vec<PathBuf>) {
        let mut t = self.compiler.value.borrow_mut();

        t.set_include_paths(include_paths);
    }

    pub fn library_paths(&self) -> HashMap<String, PathBuf> {
        let t = self.compiler.value.borrow();

        let mut paths = HashMap::new();

        for (key, path) in t.library_paths() {
            paths.insert(key.clone(), path.clone());
        }

        paths
    }

    pub fn set_library_paths(&self, library_paths: HashMap<String, PathBuf>) {
        let mut t = self.compiler.value.borrow_mut();

        t.set_library_paths(library_paths);
    }

    pub fn style(&self) -> Option<String> {
        let t =self.compiler.value.borrow();
        
        t.style().cloned()
    }

    pub fn set_style(&self, style: String) {
        let mut t = self.compiler.value.borrow_mut();
        
        t.set_style(style);
    }
}

impl CompilationResult {
    pub fn valid(&self) -> bool {
        !self.result.value.borrow().has_errors()
    }

    pub fn diagnostics(ruby: &Ruby, rb_self: &Self) -> RArray {
        let t = rb_self.result.value.borrow();

        ruby.ary_from_iter(t.diagnostics().map(|d| Diagnostic {
            diagnostic: MyWrapper::new(d)
        }))
    }

    pub fn component_names(&self) -> Vec<String> {
        let t = self.result.value.borrow();

        t
            .component_names()
            .map(|name| name.to_string())
            .collect()
    }

    pub fn components(ruby: &Ruby, rb_self: &Self) -> RArray {
        let t = rb_self.result.value.borrow();

        ruby.ary_from_iter(t.components().map(|c| ComponentDefinition {
            definition: MyWrapper::new(c)
        }))
    }
}

#[magnus::wrap(class = "Slint::Diagnostic")]
pub struct Diagnostic {
    diagnostic: MyWrapper<slint_interpreter::Diagnostic>
}

impl Diagnostic {
    pub fn level(ruby: &Ruby, rb_self: &Self) -> magnus::StaticSymbol {
        let t = rb_self.diagnostic.value.borrow();

        match t.level() {
            slint_interpreter::DiagnosticLevel::Error => ruby.sym_new("error"),
            slint_interpreter::DiagnosticLevel::Warning => ruby.sym_new("warning"),
            _ => ruby.sym_new("UNKNOWN")
        }
    }

    pub fn message(&self) -> String {
        let t = self.diagnostic.value.borrow();

        t.message().to_string()
    }

    pub fn line_column(&self) -> (usize, usize) {
        let t = self.diagnostic.value.borrow();

        t.line_column()
    }

    pub fn source_file(&self) -> Option<PathBuf> {
        let t = self.diagnostic.value.borrow();

        t.source_file().map(|sf| sf.to_owned())
    }
}

#[magnus::wrap(class = "Slint::ComponentDefinition")]
pub struct ComponentDefinition {
    definition: MyWrapper<slint_interpreter::ComponentDefinition>
}

impl ComponentDefinition {
    pub fn render(&self) {
        let t = self.definition.value.borrow();

        let instance = t.create().unwrap();
        instance.run().unwrap();
    }
}

struct MyWrapper<T> {
    value: RefCell<T>,
    thread_id: ThreadId
}

impl<T> MyWrapper<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: RefCell::new(value),
            thread_id: thread::current().id()
        }
    }
}

unsafe impl<T> Send for MyWrapper<T> {}