use slint_interpreter::{CompilationResult, Compiler, ComponentHandle};
use std::thread;
use std::sync::mpsc;

struct ActorState {
    compiler: Compiler,
    compilation_results: Vec<CompilationResult>
}

enum Message {
    BuildPath(String, mpsc::SyncSender<usize>),
    Render(usize, mpsc::SyncSender<()>)
}

#[magnus::wrap(class = "Slint::Compiler")]
pub struct CompilerWrapper {
    channel: mpsc::SyncSender<Message>,
}

impl Default for CompilerWrapper {
    fn default() -> Self {
        let (channel, recv) = mpsc::sync_channel(0);

        thread::spawn(move || {
            let state = ActorState {
                compiler: Compiler::default(),
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
            Message::BuildPath(path, reply_sender) => {
                let compilation_result = spin_on::spin_on(state.compiler.build_from_path(path));
                state.compilation_results.push(compilation_result);
                let handle = state.compilation_results.len() - 1;
                reply_sender.send(handle).unwrap();
            }
            Message::Render(handle, reply_sender) => {
                let compilation_result = state.compilation_results.get(handle).unwrap();
                if let Some(definition) = compilation_result.component("HelloWorld") {
                    let instance = definition.create().unwrap();
                    instance.run().unwrap();
                    reply_sender.send(()).unwrap();
                }
            }
        }
    }
}

impl CompilerWrapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_from_path(&self, path: String) -> CompilationResultWrapper {
        let (send, recv) = mpsc::sync_channel(0);
        self.channel.send(Message::BuildPath(path, send)).unwrap();
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
        let message = Message::Render(self.handle, send);
        self.channel.send(message).unwrap();

        recv.recv().unwrap()
    }
}