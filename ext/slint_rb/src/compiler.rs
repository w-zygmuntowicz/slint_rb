use slint_interpreter::{CompilationResult, ComponentHandle};
use std::{thread};
use std::sync::mpsc;

struct ActorState {
    compiler: slint_interpreter::Compiler,
    compilation_results: Vec<CompilationResult>
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
                compilation_results: Vec::new()
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

    pub fn build_from_path(&self, path: String) -> CompilationResultWrapper {
        let (send, recv) = mpsc::sync_channel(0);
        
        let evolution = move |state: &mut ActorState| {
            let compilation_result = spin_on::spin_on(state.compiler.build_from_path(path.clone()));
            state.compilation_results.push(compilation_result);
            let handle = state.compilation_results.len() - 1;
            send.send(handle).unwrap();
        };

        let message = Message::Evolution(Box::new(evolution));
        self.channel.send(message).unwrap();
        
        let handle = recv.recv().unwrap();
        CompilationResultWrapper {
            channel: self.channel.clone(),
            handle
        }
    }
}


#[magnus::wrap(class = "Slint::CompilationResult")]
pub struct CompilationResultWrapper {
    channel: mpsc::SyncSender<Message>,
    handle: usize
}

impl CompilationResultWrapper {
    pub fn render(&self) {
        let (send, recv) = mpsc::sync_channel(0);
        
        let index = self.handle.clone();
        let evolution = move |state: &mut ActorState| {
            let compilation_result: &CompilationResult = state.compilation_results.get(index).unwrap();

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