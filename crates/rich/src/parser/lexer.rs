//! Lexer for Rich markup.
//!
//! Converts input text into a stream of tokens.

use crate::error::RichParseError;

/// A token produced by the lexer.
#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    /// Plain text content.
    Text(&'a str),
    /// Opening/style tag content (without brackets): `bold red`
    OpenTag(&'a str),
    /// Closing tag: `None` for `[/]`, `Some("bold")` for `[/bold]`
    CloseTag(Option<&'a str>),
    /// Escaped bracket character.
    EscapedBracket(char),
}

/// Lexer for Rich markup text.
///
/// Converts input text into a stream of tokens.
///
/// # Examples
///
/// ```
/// use rich::parser::Lexer;
///
/// let mut lexer = Lexer::new("[bold]Hello[/]");
/// let tokens: Vec<_> = lexer.collect();
/// assert_eq!(tokens.len(), 3);
/// ```
pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given input.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Get the remaining input.
    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }

    /// Peek at the next character without consuming it.
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Advance by one character.
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Consume text until we hit a special character or end of input.
    fn consume_text(&mut self) -> Option<&'a str> {
        let start = self.pos;

        while let Some(c) = self.peek() {
            match c {
                '[' | '\\' => break,
                _ => {
                    self.advance();
                }
            }
        }

        if self.pos > start {
            Some(&self.input[start..self.pos])
        } else {
            None
        }
    }

    /// Consume a tag (including the brackets).
    fn consume_tag(&mut self) -> Result<Token<'a>, RichParseError> {
        let tag_start = self.pos;

        // Skip opening bracket
        self.advance(); // consume '['

        // Find the closing bracket
        let content_start = self.pos;
        let mut depth = 1;

        while depth > 0 {
            match self.peek() {
                Some('[') => {
                    depth += 1;
                    self.advance();
                }
                Some(']') => {
                    depth -= 1;
                    if depth > 0 {
                        self.advance();
                    }
                }
                Some('\\') => {
                    // Skip escape sequence
                    self.advance();
                    self.advance();
                }
                Some(_) => {
                    self.advance();
                }
                None => {
                    return Err(RichParseError::UnclosedTag(tag_start));
                }
            }
        }

        let content = &self.input[content_start..self.pos];
        self.advance(); // consume ']'

        // Determine if it's a close tag or open tag
        if content.starts_with('/') {
            let rest = content[1..].trim();
            if rest.is_empty() {
                Ok(Token::CloseTag(None))
            } else {
                Ok(Token::CloseTag(Some(rest)))
            }
        } else if content.is_empty() {
            Err(RichParseError::EmptyTag(tag_start))
        } else {
            Ok(Token::OpenTag(content))
        }
    }

    /// Consume an escape sequence.
    fn consume_escape(&mut self) -> Result<Token<'a>, RichParseError> {
        let escape_start = self.pos;
        self.advance(); // consume '\'

        match self.peek() {
            Some('[') => {
                self.advance();
                Ok(Token::EscapedBracket('['))
            }
            Some(']') => {
                self.advance();
                Ok(Token::EscapedBracket(']'))
            }
            Some('\\') => {
                self.advance();
                Ok(Token::EscapedBracket('\\'))
            }
            _ => Err(RichParseError::InvalidEscape(escape_start)),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, RichParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        match self.peek() {
            Some('[') => Some(self.consume_tag()),
            Some('\\') => Some(self.consume_escape()),
            _ => self.consume_text().map(|t| Ok(Token::Text(t))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Token<'_>> {
        Lexer::new(input).collect::<Result<Vec<_>, _>>().unwrap()
    }

    #[test]
    fn lex_plain_text() {
        let tokens = lex("Hello World");
        assert_eq!(tokens, vec![Token::Text("Hello World")]);
    }

    #[test]
    fn lex_open_tag() {
        let tokens = lex("[bold]");
        assert_eq!(tokens, vec![Token::OpenTag("bold")]);
    }

    #[test]
    fn lex_close_tag() {
        let tokens = lex("[/]");
        assert_eq!(tokens, vec![Token::CloseTag(None)]);
    }

    #[test]
    fn lex_close_tag_with_name() {
        let tokens = lex("[/bold]");
        assert_eq!(tokens, vec![Token::CloseTag(Some("bold"))]);
    }

    #[test]
    fn lex_styled_text() {
        let tokens = lex("[bold]Hello[/]");
        assert_eq!(
            tokens,
            vec![
                Token::OpenTag("bold"),
                Token::Text("Hello"),
                Token::CloseTag(None),
            ]
        );
    }

    #[test]
    fn lex_combined_style() {
        let tokens = lex("[bold red on blue]text[/]");
        assert_eq!(
            tokens,
            vec![
                Token::OpenTag("bold red on blue"),
                Token::Text("text"),
                Token::CloseTag(None),
            ]
        );
    }

    #[test]
    fn lex_nested_tags() {
        let tokens = lex("[bold][red]text[/][/]");
        assert_eq!(
            tokens,
            vec![
                Token::OpenTag("bold"),
                Token::OpenTag("red"),
                Token::Text("text"),
                Token::CloseTag(None),
                Token::CloseTag(None),
            ]
        );
    }

    #[test]
    fn lex_escaped_bracket() {
        let tokens = lex(r"\[not a tag\]");
        assert_eq!(
            tokens,
            vec![
                Token::EscapedBracket('['),
                Token::Text("not a tag"),
                Token::EscapedBracket(']'),
            ]
        );
    }

    #[test]
    fn lex_escaped_backslash() {
        let tokens = lex(r"\\");
        assert_eq!(tokens, vec![Token::EscapedBracket('\\')]);
    }

    #[test]
    fn lex_mixed_content() {
        let tokens = lex(r"Hello [bold]World[/] \[escaped\]");
        assert_eq!(
            tokens,
            vec![
                Token::Text("Hello "),
                Token::OpenTag("bold"),
                Token::Text("World"),
                Token::CloseTag(None),
                Token::Text(" "),
                Token::EscapedBracket('['),
                Token::Text("escaped"),
                Token::EscapedBracket(']'),
            ]
        );
    }

    #[test]
    fn lex_meta_tag() {
        let tokens = lex("[@click=app.quit]Exit[/]");
        assert_eq!(
            tokens,
            vec![
                Token::OpenTag("@click=app.quit"),
                Token::Text("Exit"),
                Token::CloseTag(None),
            ]
        );
    }

    #[test]
    fn lex_unclosed_tag() {
        let result: Result<Vec<_>, _> = Lexer::new("[bold").collect();
        assert!(matches!(result, Err(RichParseError::UnclosedTag(_))));
    }

    #[test]
    fn lex_empty_tag() {
        let result: Result<Vec<_>, _> = Lexer::new("[]").collect();
        assert!(matches!(result, Err(RichParseError::EmptyTag(_))));
    }

    #[test]
    fn lex_unicode() {
        let tokens = lex("[bold]日本語[/]");
        assert_eq!(
            tokens,
            vec![
                Token::OpenTag("bold"),
                Token::Text("日本語"),
                Token::CloseTag(None),
            ]
        );
    }
}
