//! Command palette widget (modal overlay).

use std::cell::RefCell;
use std::marker::PhantomData;

use tcss::types::{Layout, Visibility};
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};

use crate::canvas::{Canvas, Region};
use crate::containers::container::Container;
use crate::widget::input::Input;
use crate::widget::loading_indicator::LoadingIndicator;
use crate::widget::option_list::OptionList;
use crate::widget::static_widget::Static;
use crate::{KeyCode, MouseEvent, MouseEventKind, Size, Widget};

#[derive(Debug, Clone)]
struct Command {
    name: String,
    action: String,
    help: Option<String>,
}

fn escape_markup(input: &str) -> String {
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

fn highlight_match(name: &str, query_lower: &str) -> String {
    if query_lower.is_empty() {
        return escape_markup(name);
    }
    let name_lower = name.to_lowercase();
    if let Some(start) = name_lower.find(query_lower) {
        let end = start + query_lower.len();
        if start <= name.len() && end <= name.len() {
            let prefix = &name[..start];
            let matched = &name[start..end];
            let suffix = &name[end..];
            return format!(
                "{}[bold underline]{}[/]{}",
                escape_markup(prefix),
                escape_markup(matched),
                escape_markup(suffix)
            );
        }
    }
    escape_markup(name)
}

/// Command palette overlay widget.
pub struct CommandPalette<M: 'static> {
    inner: Container<M>,
    visible: bool,
    focused: bool,
    query: String,
    commands: Vec<Command>,
    filtered: Vec<usize>,
    selected: usize,
    pending_action: RefCell<Option<String>>,
    panel_index: usize,
    input_row_index: usize,
    input_index: usize,
    results_index: usize,
    list_index: usize,
    loading_index: usize,
    restore_focus: Option<usize>,
    _phantom: PhantomData<M>,
}

impl<M: 'static> CommandPalette<M> {
    pub fn new() -> Self {
        let search_icon = Static::new("🔎").with_id("--search-icon");
        let input = Input::new()
            .with_placeholder("Search for commands…")
            .with_id("--input-field");
        let list = OptionList::new(Vec::new()).with_markup(true);
        let loading = LoadingIndicator::new().with_id("--loading");

        let input_row = Container::new(vec![Box::new(search_icon), Box::new(input)])
            .with_id("--input")
            .with_layout(Layout::Horizontal);
        let results = Container::new(vec![Box::new(list), Box::new(loading)])
            .with_id("--results")
            .with_layout(Layout::Vertical);
        let panel = Container::new(vec![Box::new(input_row), Box::new(results)])
            .with_id("--container")
            .with_layout(Layout::Vertical);

        let children: Vec<Box<dyn Widget<M>>> = vec![Box::new(panel)];

        let inner = Container::new(children)
            .with_id("--command-palette")
            .with_classes("--textual-command-palette");

        let mut palette = Self {
            inner,
            visible: false,
            focused: false,
            query: String::new(),
            commands: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            pending_action: RefCell::new(None),
            panel_index: 0,
            input_row_index: 0,
            input_index: 1,
            results_index: 1,
            list_index: 0,
            loading_index: 1,
            restore_focus: None,
            _phantom: PhantomData,
        };
        palette.refresh_results();
        palette
    }

    pub fn open(&mut self) {
        if !self.visible {
            self.visible = true;
            self.inner.mark_dirty();
        }
    }

    pub fn open_with_focus(&mut self, focus_index: usize) {
        if !self.visible {
            self.restore_focus = Some(focus_index);
        }
        self.open();
    }

    pub fn close(&mut self) {
        if self.visible {
            self.visible = false;
            self.query.clear();
            self.selected = 0;
            self.refresh_results();
            self.inner.mark_dirty();
        }
    }

    pub fn is_open(&self) -> bool {
        self.visible
    }

    pub fn set_commands(&mut self, commands: Vec<(String, String, Option<String>)>) {
        self.commands = commands
            .into_iter()
            .map(|(name, action, help)| Command { name, action, help })
            .collect();
        self.refresh_results();
    }

    pub fn take_restore_focus(&mut self) -> Option<usize> {
        if self.visible {
            None
        } else {
            self.restore_focus.take()
        }
    }

    fn refresh_results(&mut self) {
        self.filtered.clear();
        let query_lower = self.query.to_lowercase();
        if query_lower.is_empty() {
            self.filtered.extend(0..self.commands.len());
        } else {
            for (idx, command) in self.commands.iter().enumerate() {
                if command.name.to_lowercase().contains(&query_lower) {
                    self.filtered.push(idx);
                }
            }
        }
        if self.filtered.is_empty() {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(self.filtered.len() - 1);
        }
        self.update_list_widget();
        self.update_input_widget();
        if let Some(loading) = self.loading_mut() {
            loading.set_visible(false);
        }
    }

    fn update_list_widget(&mut self) {
        let selected = self.selected;
        let query_lower = self.query.to_lowercase();
        // Description color for non-highlighted items (matches $text-muted)
        let desc_color = "#a1a5a8";

        let items: Vec<String> = self
            .filtered
            .iter()
            .map(|index| {
                let command = &self.commands[*index];
                let name = highlight_match(&command.name, &query_lower);
                let mut lines = Vec::new();
                lines.push(format!("  [bold]{name}[/]"));
                if let Some(help) = &command.help {
                    lines.push(format!("  [{desc_color}]{}[/]", escape_markup(help)));
                }
                lines.join("\n")
            })
            .collect();
        if let Some(list) = self.list_mut() {
            list.set_items(items);
            list.set_selected(selected);
        }
    }

    fn update_input_widget(&mut self) {
        let query = self.query.clone();
        if let Some(input) = self.input_mut() {
            let cursor = query.chars().count();
            input.set_value(query);
            input.set_cursor(cursor);
        }
    }

    fn input_mut(&mut self) -> Option<&mut Input<M>> {
        let index = self.input_index;
        self.input_row_mut()
            .and_then(|row| row.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Input<M>>())
    }

    fn list_mut(&mut self) -> Option<&mut OptionList<M>> {
        let index = self.list_index;
        self.results_mut()
            .and_then(|results| results.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<OptionList<M>>())
    }

    fn loading_mut(&mut self) -> Option<&mut LoadingIndicator<M>> {
        let index = self.loading_index;
        self.results_mut()
            .and_then(|results| results.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<LoadingIndicator<M>>())
    }

    fn panel_mut(&mut self) -> Option<&mut Container<M>> {
        self.inner
            .get_child_mut(self.panel_index)
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Container<M>>())
    }

    fn input_row_mut(&mut self) -> Option<&mut Container<M>> {
        let index = self.input_row_index;
        self.panel_mut()
            .and_then(|panel| panel.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Container<M>>())
    }

    fn results_mut(&mut self) -> Option<&mut Container<M>> {
        let index = self.results_index;
        self.panel_mut()
            .and_then(|panel| panel.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Container<M>>())
    }

    fn move_selection(&mut self, delta: i32) {
        if self.filtered.is_empty() {
            return;
        }
        let len = self.filtered.len() as i32;
        let next = (self.selected as i32 + delta).clamp(0, len - 1) as usize;
        if next != self.selected {
            self.selected = next;
            self.update_list_widget();
        }
    }

    fn activate_selected(&mut self) {
        if self.filtered.is_empty() {
            return;
        }
        let command_index = self.filtered[self.selected];
        let action = self.commands[command_index].action.clone();
        *self.pending_action.borrow_mut() = Some(action);
        self.close();
    }
}

impl<M: 'static> Default for CommandPalette<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> Widget<M> for CommandPalette<M> {
    fn default_css(&self) -> &'static str {
        r#"
CommandPalette {
    color: $foreground;
    background: transparent;
    align-horizontal: center;
    align-vertical: top;
    width: 100%;
    height: auto;
}

CommandPalette #--container {
    margin-top: 4;
    height: auto;
    background: $panel-darken-1;
}

CommandPalette #--search-icon {
    color: #000;
    margin-left: 1;
    margin-top: 1;
    width: 2;
}

CommandPalette #--input {
    height: auto;
    border: hkey black 50%;
}

CommandPalette #--results {
    height: auto;
    margin: 0;
    padding: 0;
    border-bottom: hkey black;
}

CommandPalette Input {
    border: blank;
    width: 1fr;
    padding-left: 0;
    background: transparent;
    background-tint: transparent 0%;
}

CommandPalette OptionList {
    height: auto;
    max-height: 70vh;
    background: transparent;
    border: none;
    padding: 0;
    line-pad: 0;
    color: $foreground;
    link-color: $block-cursor-blurred-foreground;
    link-background: $block-cursor-blurred-background;
    link-style: $block-cursor-blurred-text-style;
}

CommandPalette LoadingIndicator {
    visibility: hidden;
}

CommandPalette LoadingIndicator.--visible {
    visibility: visible;
}
"#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        if !self.visible || self.get_style().visibility == Visibility::Hidden {
            return;
        }
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "CommandPalette";
        meta.type_names = vec!["CommandPalette", "Container", "Widget", "DOMNode"];
        meta.states = self.get_state();
        meta
    }

    fn get_state(&self) -> WidgetStates {
        let mut states = WidgetStates::empty();
        if self.focused {
            states |= WidgetStates::FOCUS;
        }
        states
    }

    fn set_style(&mut self, style: ComputedStyle) {
        self.inner.set_style(style);
    }

    fn get_style(&self) -> ComputedStyle {
        self.inner.get_style()
    }

    fn set_inline_style(&mut self, style: StyleOverride) {
        self.inner.set_inline_style(style);
    }

    fn inline_style(&self) -> Option<&StyleOverride> {
        self.inner.inline_style()
    }

    fn clear_inline_style(&mut self) {
        self.inner.clear_inline_style();
    }

    fn is_dirty(&self) -> bool {
        self.inner.is_dirty()
    }

    fn mark_dirty(&mut self) {
        self.inner.mark_dirty();
    }

    fn mark_clean(&mut self) {
        self.inner.mark_clean();
    }

    fn on_event(&mut self, key: KeyCode) -> Option<M> {
        if !self.visible {
            return None;
        }
        match key {
            KeyCode::Esc => {
                self.close();
            }
            KeyCode::Enter => {
                self.activate_selected();
            }
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Backspace => {
                if !self.query.is_empty() {
                    self.query.pop();
                    self.selected = 0;
                    self.refresh_results();
                }
            }
            KeyCode::Char(ch) => {
                self.query.push(ch);
                self.selected = 0;
                self.refresh_results();
            }
            _ => {}
        }
        None
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        if !self.visible {
            return None;
        }
        let mx = event.column as i32;
        let my = event.row as i32;
        let hit_child = self.inner.hit_test_children(region, mx, my);
        if matches!(event.kind, MouseEventKind::Down(_)) && !hit_child {
            self.close();
            return None;
        }
        self.inner.on_mouse(event, region)
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        if !self.visible {
            return None;
        }
        let mx = event.column as i32;
        let my = event.row as i32;
        let hit_child = self.inner.hit_test_children(region, mx, my);
        if matches!(event.kind, MouseEventKind::Down(_)) && !hit_child {
            self.close();
            return None;
        }
        self.inner.on_mouse_with_sender(event, region)
    }

    fn take_pending_action(&self) -> Option<String> {
        self.pending_action.borrow_mut().take()
    }

    fn is_focusable(&self) -> bool {
        self.visible
    }

    fn set_focus(&mut self, is_focused: bool) {
        if self.focused != is_focused {
            self.focused = is_focused;
            if let Some(input) = self.input_mut() {
                input.set_focus(is_focused);
            }
            self.inner.mark_dirty();
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_visible(&mut self, visible: bool) {
        if self.visible != visible {
            self.visible = visible;
            self.inner.mark_dirty();
        }
    }

    fn is_loading(&self) -> bool {
        false
    }

    fn set_loading(&mut self, _loading: bool) {}

    fn is_disabled(&self) -> bool {
        false
    }

    fn set_disabled(&mut self, _disabled: bool) {}

    fn count_focusable(&self) -> usize {
        if self.is_focusable() { 1 } else { 0 }
    }

    fn clear_focus(&mut self) {
        self.focused = false;
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        if !self.is_focusable() {
            return false;
        }
        if n == 0 {
            self.set_focus(true);
            true
        } else {
            false
        }
    }

    fn child_count(&self) -> usize {
        self.inner.child_count()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut (dyn Widget<M> + '_)> {
        self.inner.get_child_mut(index)
    }

    fn handle_message(&mut self, envelope: &mut crate::MessageEnvelope<M>) -> Option<M> {
        self.inner.handle_message(envelope)
    }

    fn id(&self) -> Option<&str> {
        self.inner.id()
    }

    fn type_name(&self) -> &'static str {
        "CommandPalette"
    }

    fn on_resize(&mut self, size: Size) {
        self.inner.on_resize(size);
    }

    fn for_each_child(&mut self, f: &mut dyn FnMut(&mut dyn Widget<M>)) {
        self.inner.for_each_child(f);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn add_class(&mut self, class: &str) {
        self.inner.add_class(class);
    }

    fn remove_class(&mut self, class: &str) {
        self.inner.remove_class(class);
    }

    fn has_class(&self, class: &str) -> bool {
        self.inner.has_class(class)
    }

    fn set_classes(&mut self, classes: &str) {
        self.inner.set_classes(classes);
    }

    fn classes(&self) -> Vec<String> {
        self.inner.classes()
    }
}
