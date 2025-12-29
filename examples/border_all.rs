use textual::{App, Compose, Grid, KeyCode, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct AllBorderApp {
    quit: bool,
}

impl AllBorderApp {
    fn new() -> Self {
        Self { quit: false }
    }
}

impl Compose for AllBorderApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Grid {
                Label("ascii", id: "ascii")
                Label("blank", id: "blank")
                Label("dashed", id: "dashed")
                Label("double", id: "double")
                Label("heavy", id: "heavy")
                Label("hidden/none", id: "hidden")
                Label("hkey", id: "hkey")
                Label("inner", id: "inner")
                Label("outer", id: "outer")
                Label("panel", id: "panel")
                Label("round", id: "round")
                Label("solid", id: "solid")
                Label("tall", id: "tall")
                Label("thick", id: "thick")
                Label("vkey", id: "vkey")
                Label("wide", id: "wide")
            }
        }
    }
}

impl App for AllBorderApp {
    const CSS: &'static str = include_str!("border_all.tcss");

    fn on_key(&mut self, key: textual::KeyCode) {
        if key == KeyCode::Char('q') || key == KeyCode::Esc {
            self.quit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.quit
    }
}

fn main() -> textual::Result<()> {
    let mut app = AllBorderApp::new();
    app.run()
}
