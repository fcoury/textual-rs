//! Fuzzy matcher.
//!
//! This module provides fuzzy matching and highlighting used by the command palette.

use std::collections::HashSet;

/// Perform fuzzy matching against a candidate string.
#[derive(Debug, Clone)]
pub struct FuzzySearch {
    case_sensitive: bool,
}

impl FuzzySearch {
    /// Create a new fuzzy searcher.
    pub fn new(case_sensitive: bool) -> Self {
        Self { case_sensitive }
    }

    /// Match a query against a candidate string.
    ///
    /// Returns a score and the matched character positions (as char indices).
    pub fn match_candidate(&self, query: &str, candidate: &str) -> (f32, Vec<usize>) {
        if query.is_empty() || candidate.is_empty() {
            return (0.0, Vec::new());
        }

        let (query, candidate) = if self.case_sensitive {
            (query.to_string(), candidate.to_string())
        } else {
            (query.to_lowercase(), candidate.to_lowercase())
        };

        // Fast path: substring match.
        if let Some(byte_index) = candidate.find(&query) {
            let start = byte_to_char_index(&candidate, byte_index);
            let query_len = query.chars().count();
            let offsets: Vec<usize> = (start..start + query_len).collect();
            let score =
                self.score(&candidate, &offsets) * if candidate == query { 2.0 } else { 1.5 };
            return (score, offsets);
        }

        let candidate_chars: Vec<char> = candidate.chars().collect();
        let query_chars: Vec<char> = query.chars().collect();
        let candidate_len = candidate_chars.len();

        let mut letter_positions: Vec<Vec<usize>> = Vec::new();
        let mut position = 0usize;

        for (offset, letter) in query_chars.iter().enumerate() {
            let last_index = candidate_len.saturating_sub(offset);
            let mut positions = Vec::new();
            let mut index = position;
            while index < candidate_len {
                if candidate_chars[index] == *letter {
                    positions.push(index);
                }
                index += 1;
                if index >= last_index {
                    break;
                }
            }
            if positions.is_empty() {
                return (0.0, Vec::new());
            }
            position = positions[0] + 1;
            letter_positions.push(positions);
        }

        let mut possible_offsets: Vec<Vec<usize>> = Vec::new();
        let query_len = query_chars.len();

        fn collect_offsets(
            all_positions: &[Vec<usize>],
            query_len: usize,
            index: usize,
            offsets: &mut Vec<usize>,
            out: &mut Vec<Vec<usize>>,
        ) {
            for &pos in &all_positions[index] {
                if offsets.last().map_or(true, |last| pos > *last) {
                    offsets.push(pos);
                    if offsets.len() == query_len {
                        out.push(offsets.clone());
                    } else {
                        collect_offsets(all_positions, query_len, index + 1, offsets, out);
                    }
                    offsets.pop();
                }
            }
        }

        collect_offsets(
            &letter_positions,
            query_len,
            0,
            &mut Vec::new(),
            &mut possible_offsets,
        );

        let mut best_score = 0.0;
        let mut best_offsets = Vec::new();
        for offsets in possible_offsets {
            let score = self.score(&candidate, &offsets);
            if score > best_score {
                best_score = score;
                best_offsets = offsets;
            }
        }

        (best_score, best_offsets)
    }

    fn score(&self, candidate: &str, positions: &[usize]) -> f32 {
        if positions.is_empty() {
            return 0.0;
        }

        let first_letters = first_letter_positions(candidate);
        let offset_count = positions.len() as f32;
        let first_letter_matches = positions
            .iter()
            .filter(|pos| first_letters.contains(pos))
            .count() as f32;

        let mut score = offset_count + first_letter_matches;

        let mut groups = 1usize;
        let mut last_offset = positions[0];
        for &offset in &positions[1..] {
            if offset != last_offset + 1 {
                groups += 1;
            }
            last_offset = offset;
        }

        let normalized_groups = (offset_count - (groups as f32 - 1.0)) / offset_count;
        score *= 1.0 + (normalized_groups * normalized_groups);
        score
    }
}

/// A fuzzy matcher with highlighting.
#[derive(Debug, Clone)]
pub struct Matcher {
    query: String,
    match_style: String,
    fuzzy: FuzzySearch,
}

impl Matcher {
    /// Create a new matcher.
    pub fn new(
        query: impl Into<String>,
        match_style: Option<String>,
        case_sensitive: bool,
    ) -> Self {
        let query = query.into();
        let match_style = match_style.unwrap_or_else(|| "reverse".to_string());
        Self {
            query,
            match_style,
            fuzzy: FuzzySearch::new(case_sensitive),
        }
    }

    /// Return the match score for a candidate string.
    pub fn match_score(&self, candidate: &str) -> f32 {
        self.fuzzy.match_candidate(&self.query, candidate).0
    }

    /// Highlight matched characters in a candidate string with markup.
    pub fn highlight(&self, candidate: &str) -> String {
        let (score, offsets) = self.fuzzy.match_candidate(&self.query, candidate);
        if score <= 0.0 {
            return escape_markup(candidate);
        }

        let offset_set: HashSet<usize> = offsets.into_iter().collect();
        let mut out = String::with_capacity(candidate.len());
        for (index, ch) in candidate.chars().enumerate() {
            let mut escaped = String::new();
            push_escaped_char(&mut escaped, ch);
            if offset_set.contains(&index) && !ch.is_whitespace() && !self.match_style.is_empty() {
                out.push('[');
                out.push_str(&self.match_style);
                out.push(']');
                out.push_str(&escaped);
                out.push_str("[/]");
            } else {
                out.push_str(&escaped);
            }
        }
        out
    }
}

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn first_letter_positions(candidate: &str) -> HashSet<usize> {
    let mut positions = HashSet::new();
    let mut in_word = false;
    for (index, ch) in candidate.chars().enumerate() {
        let word = is_word_char(ch);
        if word && !in_word {
            positions.insert(index);
        }
        in_word = word;
    }
    positions
}

fn byte_to_char_index(s: &str, byte_index: usize) -> usize {
    s[..byte_index].chars().count()
}

fn escape_markup(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        push_escaped_char(&mut out, ch);
    }
    out
}

fn push_escaped_char(out: &mut String, ch: char) {
    match ch {
        '[' | ']' | '\\' => {
            out.push('\\');
            out.push(ch);
        }
        _ => out.push(ch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzzy_match_substring_scores() {
        let search = FuzzySearch::new(false);
        let (score, offsets) = search.match_candidate("test", "this is a test");
        assert!(score > 0.0);
        assert_eq!(offsets.len(), 4);
    }

    #[test]
    fn matcher_highlight_applies_markup() {
        let matcher = Matcher::new("tp", Some("bold".to_string()), false);
        let highlighted = matcher.highlight("text palette");
        assert!(highlighted.contains("[bold]"));
    }
}
