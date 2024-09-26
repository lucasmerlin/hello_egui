use egui_inbox::{AsRequestRepaint, RequestRepaintContext, UiInbox};

pub struct MyApplicationState {
    state: Option<String>,
    inbox: UiInbox<String>,
    repaint_rx: std::sync::mpsc::Receiver<()>,
    repaint_tx: std::sync::mpsc::Sender<()>,
}

impl AsRequestRepaint for MyApplicationState {
    fn as_request_repaint(&self) -> RequestRepaintContext {
        let repaint_tx = self.repaint_tx.clone();
        RequestRepaintContext::from_callback(move || {
            repaint_tx.send(()).unwrap();
        })
    }
}

impl Default for MyApplicationState {
    fn default() -> Self {
        let (repaint_tx, repaint_rx) = std::sync::mpsc::channel();
        Self {
            state: None,
            inbox: UiInbox::new(),
            repaint_rx,
            repaint_tx,
        }
    }
}

impl MyApplicationState {
    pub fn run(mut self) {
        let sender = self.inbox.sender();
        std::thread::spawn(move || {
            let mut count = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                count += 1;
                sender.send(format!("Count: {count}")).ok();
            }
        });

        loop {
            self.inbox.read(&self).for_each(|msg| {
                self.state = Some(msg);
            });

            println!("State: {:?}", self.state);

            self.repaint_rx.recv().unwrap();
        }
    }
}

fn main() {
    MyApplicationState::default().run();
}
