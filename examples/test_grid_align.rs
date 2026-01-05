use textual::testing::render_to_canvas;
use textual::{Container, Grid, Label, Widget, ui};

#[derive(Clone)]
enum Message {}

struct TestApp;

fn main() {
    // Clear the debug log
    let _ = std::fs::remove_file("/tmp/grid_debug.log");

    let css = r#"
Grid {
    grid-size: 3 3;
    align: center middle;
}

Container {
    width: 100%;
    height: 100%;
    align: center middle;
    border: solid white;
}
"#;

    let canvas = render_to_canvas(&TestApp, css, 80, 24);
    println!("{}", canvas.to_snapshot());

    // Print the debug log
    if let Ok(log) = std::fs::read_to_string("/tmp/grid_debug.log") {
        println!("\n--- Debug Log ---");
        println!("{}", log);
    }
}
