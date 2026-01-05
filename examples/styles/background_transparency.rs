use textual::{App, Compose, Static, Widget, ui};

#[derive(Clone)]
enum Message {}

struct BackgroundTransparencyApp;

impl Compose for BackgroundTransparencyApp {
    type Message = Message;

    fn compose(&self) -> Vec<Box<dyn Widget<Self::Message>>> {
        ui! {
            Static("10%", id: "t10")
            Static("20%", id: "t20")
            Static("30%", id: "t30")
            Static("40%", id: "t40")
            Static("50%", id: "t50")
            Static("60%", id: "t60")
            Static("70%", id: "t70")
            Static("80%", id: "t80")
            Static("90%", id: "t90")
            Static("100%", id: "t100")
        }
    }
}

impl App for BackgroundTransparencyApp {
    const CSS: &'static str = include_str!("background_transparency.tcss");
}

fn main() -> textual::Result<()> {
    let mut app = BackgroundTransparencyApp;
    app.run()
}
