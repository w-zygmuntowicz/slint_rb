use slint_interpreter::{Compiler, ComponentHandle};
use std::thread;
use std::sync::mpsc::{self, SyncSender};

enum Message {
    BuildPath(String, mpsc::SyncSender<()>)
}

#[magnus::wrap(class = "Slint::Compiler")]
pub struct CompilerWrapper {
    channel: SyncSender<Message>,
}

impl CompilerWrapper {
    pub fn new() -> Self {
        let (channel, recv) = mpsc::sync_channel(0);
        thread::spawn(move || {
            let compiler = Compiler::default();

            while let Ok(msg) = recv.recv() {
                match msg {
                    Message::BuildPath(path, rtrn ) => {
                        let result = spin_on::spin_on(compiler.build_from_path(path));
                        if let Some(definition) = result.component("HelloWorld") {
                            let instance = definition.create().unwrap();
                            instance.run().unwrap();
                        }

                        rtrn.send(()).unwrap()
                    },
                }
            }
        });

        Self { channel }
    }

    pub fn build_from_path(&self, path: String) {
        let (send, recv) = mpsc::sync_channel(0);
        self.channel.send(Message::BuildPath(path, send)).unwrap();
        recv.recv().unwrap()
    }
}