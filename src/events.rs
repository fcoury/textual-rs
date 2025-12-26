pub enum Message {
    SwitchChanged { id: &'static str, on: bool },
    Quit,
}
