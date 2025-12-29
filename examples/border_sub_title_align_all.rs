use textual::{App, Compose, Container, Grid, KeyCode, Label, Widget};

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
        vec![Box::new(Grid::new(vec![
            make_label_container(
                // (1)!
                "This is the story of",
                "lbl1",
                "[b]Border [i]title[/i][/]",
                "[u][r]Border[/r] subtitle[/]",
            ),
            make_label_container(
                // (2)!
                "a Python",
                "lbl2",
                "[b red]Left, but it's loooooooooooong",
                "[reverse]Center, but it's loooooooooooong",
            ),
            make_label_container(
                // (3)!
                "developer that",
                "lbl3",
                "[b i on purple]Left[/]",
                "[r u white on black]@@@[/]",
            ),
            make_label_container(
                "had to fill up",
                "lbl4",
                "",                                              // (4)!
                "[link='https://textual.textualize.io']Left[/]", // (5)!
            ),
            make_label_container(
                // (6)!
                "nine labels",
                "lbl5",
                "Title",
                "Subtitle",
            ),
            make_label_container(
                // (7)!
                "and ended up redoing it",
                "lbl6",
                "Title",
                "Subtitle",
            ),
            make_label_container(
                // (8)!
                "because the first try",
                "lbl7",
                "Title, but really loooooooooong!",
                "Subtitle, but really loooooooooong!",
            ),
            make_label_container(
                // (9)!
                "had some labels",
                "lbl8",
                "Title, but really loooooooooong!",
                "Subtitle, but really loooooooooong!",
            ),
            make_label_container(
                "that were too long.",
                "lbl9",
                "Title, but really loooooooooong!",
                "Subtitle, but really loooooooooong!",
            ),
        ]))]
    }
}

fn make_label_container<M: 'static>(
    text: &str,
    id: &str,
    border_title: &str,
    border_subtitle: &str,
) -> Box<Container<M>> {
    let label = Label::new(text)
        .with_id(id)
        .with_border_title(border_title)
        .with_border_subtitle(border_subtitle);

    Box::new(Container::new(vec![Box::new(label)]))
}

impl App for AllBorderApp {
    const CSS: &'static str = include_str!("border_sub_title_align_all.tcss");

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
