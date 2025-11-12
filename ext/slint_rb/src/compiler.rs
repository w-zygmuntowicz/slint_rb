use slint_interpreter::{ComponentHandle};
use std::path::PathBuf;
use std::thread;
use std::sync::{mpsc, Arc};
use std::collections::HashMap;
use magnus::{RArray, Ruby};

struct ActorState {
    compiler: slint_interpreter::Compiler,
    compilation_results: HashMap<usize, slint_interpreter::CompilationResult>,
    next_compilation_result_id: usize,
    diagnostics: HashMap<usize, slint_interpreter::Diagnostic>,
    next_diagnostic_id: usize,
    component_definitions: HashMap<usize, slint_interpreter::ComponentDefinition>,
    next_component_definition_id: usize
}

enum Message {
    Evolution(Box<dyn Fn(&mut ActorState) -> () + Send>)
}

struct Actor {
    channel: mpsc::SyncSender<Message>
}

impl Default for Actor {
    fn default() -> Self {
        let (channel, recv) = mpsc::sync_channel(0);

        thread::spawn(move || {
            let state = ActorState {
                compiler: slint_interpreter::Compiler::default(),
                compilation_results: HashMap::new(),
                next_compilation_result_id: 0,
                diagnostics: HashMap::new(),
                next_diagnostic_id: 0,
                component_definitions: HashMap::new(),
                next_component_definition_id: 0
            };
            actor_loop(state, recv);
        });

        Self { channel }
    }
}

impl Actor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply<F, T>(&self, f: F) -> T
    where 
        F: Fn(&mut ActorState) -> T + Send + 'static,
        T: Send + 'static
    {
        let (send, recv) = mpsc::sync_channel(0);
        let wrapper = move |state: &mut ActorState| {
            let result = f(state);
            send.send(result).unwrap();
        };
        let boxed = Box::new(wrapper);
        let message = Message::Evolution(boxed);
        self.channel.send(message).unwrap();

        recv.recv().unwrap()
    }
}

fn actor_loop(mut state: ActorState, recv: mpsc::Receiver<Message>) {
    while let Ok(msg) = recv.recv() {
        match msg {
            Message::Evolution(boxed_closure) => {
                (&boxed_closure)(&mut state);
            }
        }
    }
}

#[magnus::wrap(class = "Slint::Compiler")]
pub struct Compiler {
    actor: Arc<Actor>
}

impl Default for Compiler {
    fn default() -> Self {
        Self { actor: Arc::new(Actor::new()) }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_from_path(&self, path: String) -> CompilationResult {        
        let handle = self.actor.apply(move |state: &mut ActorState| {
            let compilation_result = spin_on::spin_on(state.compiler.build_from_path(path.clone()));
            let handle = state.next_compilation_result_id;
            state.compilation_results.insert(handle, compilation_result);
            state.next_compilation_result_id += 1;
            handle
        });

        CompilationResult {
            actor: Arc::clone(&self.actor),
            handle
        }
    }

    pub fn build_from_source(&self, source: String, path: PathBuf) -> CompilationResult {
        let handle = self.actor.apply(move |state| {
            let compilation_result = spin_on::spin_on(state.compiler.build_from_source(source.clone(), path.clone()));
            let handle = state.next_compilation_result_id;
            state.compilation_results.insert(handle, compilation_result);
            state.next_compilation_result_id += 1;
            handle
        });

        CompilationResult { actor: Arc::clone(&self.actor), handle }
    }

    pub fn include_paths(&self) -> Vec<String> {
        self.actor.apply(move |state: &mut ActorState| {
            state.compiler
                 .include_paths()
                 .iter()
                 .map(|p| p.to_str().unwrap_or_default().to_string())
                 .collect()
        })
    }

    pub fn set_include_paths(&self, include_paths: Vec<PathBuf>) {
        self.actor.apply(move |state: &mut ActorState| state.compiler.set_include_paths(include_paths.clone()))
    }

    pub fn library_paths(&self) -> HashMap<String, PathBuf> {
        self.actor.apply(move |state: &mut ActorState| {
            let mut paths = HashMap::new();

            for (key, path) in state.compiler.library_paths() {
                paths.insert(key.clone(), path.clone());
            }

            paths
        })
    }

    pub fn set_library_paths(&self, library_paths: HashMap<String, PathBuf>) {
        self.actor.apply(move |state: &mut ActorState| state.compiler.set_library_paths(library_paths.clone()))
    }

    pub fn style(&self) -> Option<String> {
        self.actor.apply(move |state: &mut ActorState| state.compiler.style().cloned())
    }

    pub fn set_style(&self, style: String) {
        self.actor.apply(move |state: &mut ActorState| state.compiler.set_style(style.clone()))
    }
}


#[magnus::wrap(class = "Slint::CompilationResult")]
pub struct CompilationResult {
    actor: Arc<Actor>,
    handle: usize
}

impl CompilationResult {
    pub fn valid(&self) -> bool {
        let index = self.handle.clone();

        self.actor.apply(move |state| {
            let compilation_result = state.compilation_results.get(&index).unwrap();

            !compilation_result.has_errors()
        })
    }

    pub fn diagnostics(ruby: &Ruby, rb_self: &Self) -> RArray {
        let index = rb_self.handle.clone();
        
        let diagnostic_handles = rb_self.actor.apply(move |state| {
            let compilation_result = state.compilation_results.get(&index).unwrap();
            let mut diagnostic_handles: Vec<usize> = Vec::new();

            compilation_result.diagnostics().for_each(|diagnostic|  {
                state.diagnostics.insert(state.next_diagnostic_id, diagnostic);
                diagnostic_handles.push(state.next_diagnostic_id);
                state.next_diagnostic_id += 1;
            });

            diagnostic_handles
        });

        ruby.ary_from_iter(diagnostic_handles.into_iter().map(|handle| Diagnostic { handle, actor: Arc::clone(&rb_self.actor) }))
    }

    pub fn component_names(&self) -> Vec<String> {
        let index = self.handle.clone();

        self.actor.apply(move |state| {
            let compilation_result = state.compilation_results.get(&index).unwrap();

            compilation_result
                .component_names()
                .map(|name| name.to_string())
                .collect()
        })
    }

    pub fn components(ruby: &Ruby, rb_self: &Self) -> RArray {
        let index = rb_self.handle.clone();

        let component_handles = rb_self.actor.apply(move |state| {
            let compilation_result = state.compilation_results.get(&index).unwrap();
            let mut component_handles: Vec<usize> = Vec::new();

            compilation_result.components().for_each(|component| {
                state.component_definitions.insert(state.next_component_definition_id, component);
                component_handles.push(state.next_component_definition_id);
                state.next_component_definition_id += 1;
            });

            component_handles
        });

        let component_definitions = component_handles
            .into_iter()
            .map(|handle| ComponentDefinition { handle, actor: Arc::clone(&rb_self.actor) });
        ruby.ary_from_iter(component_definitions)
    }
}

impl Drop for CompilationResult {
    fn drop(&mut self) {
        let index = self.handle.clone();
        
        self.actor.apply(move |state: &mut ActorState| { 
            state.compilation_results.remove(&index); 
        })
    }
}

#[magnus::wrap(class = "Slint::Diagnostic")]
pub struct Diagnostic {
    actor: Arc<Actor>,
    handle: usize
}

impl Diagnostic {
    pub fn level(ruby: &Ruby, rb_self: &Self) -> magnus::StaticSymbol {
        let index = rb_self.handle.clone();
        let level = rb_self.actor.apply(move |state| {
            let diagnostic = state.diagnostics.get(&index).unwrap();

            diagnostic.level()
        });


        match level {
            slint_interpreter::DiagnosticLevel::Error => ruby.sym_new("error"),
            slint_interpreter::DiagnosticLevel::Warning => ruby.sym_new("warning"),
            _ => todo!(),
        }
    }

    pub fn message(&self) -> String {
        let index = self.handle.clone();

        self.actor.apply(move |state| {
            let diagnostic = state.diagnostics.get(&index).unwrap();

            diagnostic.message().to_string()
        })
    }

    pub fn line_column(&self) -> (usize, usize) {
        let index = self.handle.clone();

        self.actor.apply(move |state| {
            let diagnostic = state.diagnostics.get(&index).unwrap();

            diagnostic.line_column()
        })
    }

    pub fn source_file(&self) -> Option<PathBuf> {
        let index = self.handle.clone();

        self.actor.apply(move |state| {
            let diagnostic = state.diagnostics.get(&index).unwrap();

            diagnostic.source_file().map(|path| path.to_owned())
        })
    }
}

#[magnus::wrap(class="Slint::ComponentDefinition")]
pub struct ComponentDefinition {
    handle: usize,
    actor: Arc<Actor>
}

impl ComponentDefinition {
    pub fn render(&self) {
        let index = self.handle.clone();

        self.actor.apply(move |state: &mut ActorState| {
            let component_definition = state.component_definitions.get(&index).unwrap();

            let instance = component_definition.create().unwrap();
            instance.run().unwrap();
        })
    }
}
