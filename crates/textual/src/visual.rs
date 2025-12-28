#[derive(Debug, Clone)]
pub enum VisualType {
    Text(String),
    // Styled(StyledText),
    // Rich(Box<dyn Renderable>),
}

impl From<&str> for VisualType {
    fn from(s: &str) -> Self {
        VisualType::Text(s.to_string())
    }
}

impl From<String> for VisualType {
    fn from(s: String) -> Self {
        VisualType::Text(s)
    }
}
