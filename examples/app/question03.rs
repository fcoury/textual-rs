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

    const CSS: &'static str = r#"
    Screen {
        layout: grid;
        grid-size: 2;
        grid-gutter: 2;
        padding: 2;
    }
    #question {
        width: 100%;
        height: 100%;
        column-span: 2;
        content-align: center bottom;
        text-style: bold;
    }

    Button {
        width: 100%;
    }
    "#;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Label("Do you love Textual?", id: "question")
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
