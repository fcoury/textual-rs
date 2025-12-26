use thiserror::Error;

#[derive(Error, Debug)]
pub enum TextualError {
    #[error("Terminal error: {0}")]
    IO(#[from] std::io::Error),

    #[error("CSS Parse Error: {0}")]
    InvalidCss(String),

    #[error("Layout error: Widget {0} is too large for the allocated region")]
    LayoutOverflow(String),

    #[error("The application was already running")]
    AlreadyRunning,
}

// Create a type alias for convenience
pub type Result<T> = std::result::Result<T, TextualError>;
