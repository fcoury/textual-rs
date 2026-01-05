use textual::{App, Button, EventContext, Label, MessageEnvelope, Widget, ui};

#[derive(Clone)]
enum Message {
    Pressed,
}

struct QuestionApp {
    answer: Option<String>,
}

impl QuestionApp {
    fn new() -> Self {
        Self { answer: None }
    }
}

impl App for QuestionApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Do you love Textual?")
            Button("Yes", id: "yes", variant: "primary", message: Message::Pressed)
            Button("No", id: "no", variant: "error", message: Message::Pressed)
        }
    }

    fn handle_message(
        &mut self,
        envelope: MessageEnvelope<Self::Message>,
        _ctx: &mut EventContext<Self::Message>,
    ) {
        match envelope.message {
            Message::Pressed => {
                self.answer = envelope.sender_id.clone();
                self.request_quit();
            }
        }
    }
}

fn main() -> textual::Result<()> {
    let mut app = QuestionApp::new();
    app.run()?;

    if let Some(answer) = app.answer {
        println!("{answer}");
    }
    Ok(())
}
