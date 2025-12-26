use textual::{
    App, Compose, KeyCode, Result, Switch, Widget,
    containers::{Center, Middle},
    ui,
};

struct SwitchTestApp {
    should_exit: bool,
}

impl SwitchTestApp {
    fn new() -> Self {
        Self { should_exit: false }
    }
}

impl Compose for SwitchTestApp {
    // Replicating the compose() method
    fn compose(&self) -> Box<dyn Widget + 'static> {
        // We use a macro (ui!) to handle the nesting of components
        ui! {
            Middle {
                Center {
                    Switch::new()
                }
            }
        }
    }
}

impl App for SwitchTestApp {
    // Replicating the CSS property
    const CSS: &'static str = "
        Screen {
            align: center middle;
        }
    ";

    // Replicating the on_key handler
    fn on_key(&mut self, key: KeyCode) {
        if key == KeyCode::Char('q') {
            self.should_exit = true;
        }
    }

    fn should_quit(&self) -> bool {
        self.should_exit
    }
}

fn main() -> Result<()> {
    let mut app = SwitchTestApp::new();
    app.run()
}
