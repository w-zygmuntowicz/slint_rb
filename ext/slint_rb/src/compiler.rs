use slint_interpreter::{ComponentHandle};
use std::path::PathBuf;
use std::{thread};
use std::sync::mpsc;
use std::collections::HashMap;

struct ActorState {
    compiler: slint_interpreter::Compiler,
    compilation_results: HashMap<usize, slint_interpreter::CompilationResult>,
    next_compilation_result_id: usize
}

enum Message {
    Evolution(Box<dyn Fn(&mut ActorState) -> () + Send>)
}

#[magnus::wrap(class = "Slint::Compiler")]
pub struct Compiler {
    channel: mpsc::SyncSender<Message>,
}

impl Default for Compiler {
    fn default() -> Self {
        let (channel, recv) = mpsc::sync_channel(0);

        thread::spawn(move || {
            let state = ActorState {
                compiler: slint_interpreter::Compiler::default(),
                compilation_results: HashMap::new(),
                next_compilation_result_id:  0
            };
            actor_loop(state, recv);
        });

        Self { channel }
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

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_from_path(&self, path: String) -> CompilationResult {
        let (send, recv) = mpsc::sync_channel(0);
        
        let evolution = move |state: &mut ActorState| {
            let compilation_result = spin_on::spin_on(state.compiler.build_from_path(path.clone()));
            let handle = state.next_compilation_result_id;
            state.compilation_results.insert(handle, compilation_result);
            state.next_compilation_result_id += 1;
            send.send(handle).unwrap();
        };

        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();
        
        let handle = recv.recv().unwrap();
        CompilationResult {
            channel: self.channel.clone(),
            handle
        }
    }

    pub fn include_paths(&self) -> Vec<String> {
        let (send, recv) = mpsc::sync_channel(0);

        let evolution = move |state: &mut ActorState| {
            let paths = state.compiler
                             .include_paths()
                             .iter()
                             .map(|p| p.to_str().unwrap_or_default().to_string())
                             .collect();
            
            send.send(paths).unwrap();
        };

        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();

        recv.recv().unwrap()
    }

    pub fn set_include_paths(&self, include_paths: Vec<PathBuf>) {
        let (send, recv) = mpsc::sync_channel(0);

        let evolution = move |state: &mut ActorState| {
            state.compiler.set_include_paths(include_paths.clone());

            send.send(()).unwrap();
        };
        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();

        recv.recv().unwrap();
    }

    pub fn library_paths(&self) -> HashMap<String, PathBuf> {
        let (send, recv) = mpsc::sync_channel(0);

        let evolution = move |state: &mut ActorState| {
            let mut paths = HashMap::new();

            for (key, path) in state.compiler.library_paths() {
                paths.insert(key.clone(), path.clone());
            }

            send.send(paths).unwrap();
        };
        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();

        recv.recv().unwrap()
    }

    pub fn set_library_paths(&self, library_paths: HashMap<String, PathBuf>) {
        let (send, recv) = mpsc::sync_channel(0);

        let evolution = move |state: &mut ActorState| {
            state.compiler.set_library_paths(library_paths.clone());

            send.send(()).unwrap();
        };
        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();

        recv.recv().unwrap();
    }
}


#[magnus::wrap(class = "Slint::CompilationResult")]
pub struct CompilationResult {
    channel: mpsc::SyncSender<Message>,
    handle: usize
}

impl CompilationResult {
    pub fn render(&self) {
        let (send, recv) = mpsc::sync_channel(0);
        
        let index = self.handle.clone();
        let evolution = move |state: &mut ActorState| {
            let compilation_result = state.compilation_results.get(&index).unwrap();

            if let Some(definition) = compilation_result.component("HelloWorld") {
                let instance = definition.create().unwrap();
                instance.run().unwrap();
                send.send(()).unwrap();
            }
        };

        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();

        recv.recv().unwrap()
    }
}

impl Drop for CompilationResult {
    fn drop(&mut self) {
        let (send, recv) = mpsc::sync_channel(0);

        let index = self.handle.clone();
        let evolution = move |state: &mut ActorState| {
            state.compilation_results.remove(&index);
            send.send(()).unwrap();
        };
        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();

        recv.recv().unwrap()
    }
}