use unicode_display_width::width as display_width_impl;
use unicode_segmentation::UnicodeSegmentation;

pub(crate) fn display_width(text: &str) -> usize {
    display_width_impl(text) as usize
}

pub(crate) fn grapheme_count(text: &str) -> usize {
    UnicodeSegmentation::graphemes(text, true).count()
}

pub(crate) fn grapheme_indices(text: &str) -> impl Iterator<Item = (usize, &str)> {
    UnicodeSegmentation::grapheme_indices(text, true)
}

pub(crate) fn graphemes(text: &str) -> impl Iterator<Item = &str> {
    UnicodeSegmentation::graphemes(text, true)
}

pub(crate) fn grapheme_byte_index(text: &str, grapheme_index: usize) -> usize {
    UnicodeSegmentation::grapheme_indices(text, true)
        .nth(grapheme_index)
        .map(|(idx, _)| idx)
        .unwrap_or(text.len())
}

pub(crate) fn grapheme_byte_range(text: &str, grapheme_index: usize) -> Option<(usize, usize)> {
    let mut iter = UnicodeSegmentation::grapheme_indices(text, true);
    let (start, _) = iter.nth(grapheme_index)?;
    let end = iter.next().map(|(idx, _)| idx).unwrap_or(text.len());
    Some((start, end))
}
