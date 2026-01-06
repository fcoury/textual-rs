//! Command palette core types.

use async_trait::async_trait;

use crate::fuzzy::Matcher;

/// Escape markup control characters so user strings render literally.
pub(crate) fn escape_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '[' | ']' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// A system command exposed in the command palette.
#[derive(Debug, Clone)]
pub struct SystemCommand {
    pub title: String,
    pub action: String,
    pub help: Option<String>,
    pub discover: bool,
}

impl SystemCommand {
    pub fn new(title: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: action.into(),
            help: None,
            discover: true,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn discoverable(mut self, discover: bool) -> Self {
        self.discover = discover;
        self
    }
}

impl From<(String, String, Option<String>)> for SystemCommand {
    fn from(value: (String, String, Option<String>)) -> Self {
        let (title, action, help) = value;
        Self {
            title,
            action,
            help,
            discover: true,
        }
    }
}

/// A command palette hit from a provider search.
#[derive(Debug, Clone)]
pub struct Hit {
    pub score: f32,
    pub match_display: String,
    pub action: String,
    pub text: String,
    pub help: Option<String>,
}

/// A discovery hit shown when the query is empty.
#[derive(Debug, Clone)]
pub struct DiscoveryHit {
    pub display: String,
    pub action: String,
    pub text: String,
    pub help: Option<String>,
}

/// Either a search hit or a discovery hit.
#[derive(Debug, Clone)]
pub enum CommandHit {
    Search(Hit),
    Discovery(DiscoveryHit),
}

/// Metadata for a highlighted command palette entry.
#[derive(Debug, Clone)]
pub struct CommandPaletteHighlight {
    pub index: usize,
    pub text: String,
    pub action: String,
    pub help: Option<String>,
}

/// Events emitted by the command palette.
#[derive(Debug, Clone)]
pub enum CommandPaletteEvent {
    Opened,
    Closed { option_selected: bool },
    OptionHighlighted(CommandPaletteHighlight),
}

/// Base trait for command palette providers.
#[async_trait]
pub trait Provider: Send {
    async fn startup(&mut self) {}

    async fn search(&mut self, query: &str) -> Vec<Hit>;

    async fn discover(&mut self) -> Vec<DiscoveryHit> {
        Vec::new()
    }

    async fn shutdown(&mut self) {}

    fn set_match_style(&mut self, _style: String) {}
}

/// A simple command definition.
#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub name: String,
    pub action: String,
    pub help: Option<String>,
    pub discover: bool,
}

impl SimpleCommand {
    pub fn new(name: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            action: action.into(),
            help: None,
            discover: true,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn discoverable(mut self, discover: bool) -> Self {
        self.discover = discover;
        self
    }
}

impl From<SystemCommand> for SimpleCommand {
    fn from(command: SystemCommand) -> Self {
        Self {
            name: command.title,
            action: command.action,
            help: command.help,
            discover: command.discover,
        }
    }
}

impl From<(String, String, Option<String>)> for SimpleCommand {
    fn from(value: (String, String, Option<String>)) -> Self {
        let (name, action, help) = value;
        Self {
            name,
            action,
            help,
            discover: true,
        }
    }
}

/// Provider that searches a static list of commands.
#[derive(Debug, Clone)]
pub struct SimpleProvider {
    commands: Vec<SimpleCommand>,
    match_style: Option<String>,
}

impl SimpleProvider {
    pub fn new(commands: Vec<SimpleCommand>) -> Self {
        Self {
            commands,
            match_style: None,
        }
    }
}

#[async_trait]
impl Provider for SimpleProvider {
    async fn search(&mut self, query: &str) -> Vec<Hit> {
        let matcher = Matcher::new(query, self.match_style.clone(), false);
        let mut hits = Vec::new();
        for command in &self.commands {
            let score = matcher.match_score(&command.name);
            if score > 0.0 {
                hits.push(Hit {
                    score,
                    match_display: matcher.highlight(&command.name),
                    action: command.action.clone(),
                    text: command.name.clone(),
                    help: command.help.clone(),
                });
            }
        }
        hits
    }

    async fn discover(&mut self) -> Vec<DiscoveryHit> {
        self.commands
            .iter()
            .filter(|command| command.discover)
            .map(|command| DiscoveryHit {
                display: escape_markup(&command.name),
                action: command.action.clone(),
                text: command.name.clone(),
                help: command.help.clone(),
            })
            .collect()
    }

    fn set_match_style(&mut self, style: String) {
        self.match_style = Some(style);
    }
}

/// Provider for system commands.
#[derive(Debug, Clone)]
pub struct SystemCommandsProvider {
    commands: Vec<SystemCommand>,
    match_style: Option<String>,
}

impl SystemCommandsProvider {
    pub fn new(commands: Vec<SystemCommand>) -> Self {
        Self {
            commands,
            match_style: None,
        }
    }
}

#[async_trait]
impl Provider for SystemCommandsProvider {
    async fn search(&mut self, query: &str) -> Vec<Hit> {
        let matcher = Matcher::new(query, self.match_style.clone(), false);
        let mut hits = Vec::new();
        for command in &self.commands {
            let score = matcher.match_score(&command.title);
            if score > 0.0 {
                hits.push(Hit {
                    score,
                    match_display: matcher.highlight(&command.title),
                    action: command.action.clone(),
                    text: command.title.clone(),
                    help: command.help.clone(),
                });
            }
        }
        hits
    }

    async fn discover(&mut self) -> Vec<DiscoveryHit> {
        let mut commands: Vec<SystemCommand> = self
            .commands
            .iter()
            .filter(|command| command.discover)
            .cloned()
            .collect();
        commands.sort_by(|a, b| a.title.cmp(&b.title));
        commands
            .into_iter()
            .map(|command| DiscoveryHit {
                display: escape_markup(&command.title),
                action: command.action,
                text: command.title,
                help: command.help,
            })
            .collect()
    }

    fn set_match_style(&mut self, style: String) {
        self.match_style = Some(style);
    }
}
