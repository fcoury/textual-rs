// command_palette.rs
//! Command palette widget (modal overlay).
//!
//! This file is intended to be a pixel-parity Rust port of Textual's
//! `textual.command.CommandPalette`.
//!
//! Key invariants for the port:
//! - Widget hierarchy and IDs match Python.
//! - DEFAULT_CSS is byte-identical to Python Textual for:
//!   - SearchIcon
//!   - CommandInput
//!   - CommandList
//!   - CommandPalette

use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tcss::types::{Layout, Visibility};
use tcss::{ComputedStyle, StyleOverride, WidgetMeta, WidgetStates};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::canvas::{Canvas, Region};
use crate::command::{
    CommandHit, CommandPaletteEvent, CommandPaletteHighlight, DiscoveryHit, Hit, Provider,
    SimpleCommand, SimpleProvider, escape_markup,
};
use crate::containers::container::Container;
use crate::grapheme::grapheme_count;
use crate::widget::button::Button;
use crate::widget::input::Input;
use crate::widget::loading_indicator::LoadingIndicator;
use crate::widget::option_list::{OptionItem, OptionList};
use crate::widget::static_widget::Static;
use crate::{KeyCode, MouseEvent, MouseEventKind, Size, Widget};

/// Widget for displaying a search icon before the command input.
///
/// Python: `class SearchIcon(Static, inherit_css=False)`
struct SearchIcon<M: 'static> {
    inner: Static<M>,
}

impl<M: 'static> SearchIcon<M> {
    fn new() -> Self {
        Self {
            inner: Static::new("🔎"),
        }
    }
}

impl<M: 'static> Widget<M> for SearchIcon<M> {
    fn default_css(&self) -> &'static str {
        // From Python Textual: textual.command.SearchIcon.DEFAULT_CSS
        r#"
    SearchIcon {
        color: #000;  /* required for snapshot tests */
        margin-left: 1;
        margin-top: 1;
        width: 2;
    }
    "#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "SearchIcon";
        meta.type_names = vec!["SearchIcon", "Static", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
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
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        self.inner.on_mouse_with_sender(event, region)
    }

    fn take_pending_action(&self) -> Option<String> {
        self.inner.take_pending_action()
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
    }

    fn set_focus(&mut self, is_focused: bool) {
        self.inner.set_focus(is_focused);
    }

    fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.inner.set_visible(visible);
    }

    fn is_loading(&self) -> bool {
        self.inner.is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.inner.set_loading(loading);
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled);
    }

    fn count_focusable(&self) -> usize {
        self.inner.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.inner.clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.inner.focus_nth(n)
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
        "SearchIcon"
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

/// The command palette input control.
///
/// Python: `class CommandInput(Input)`
struct CommandInput<M: 'static> {
    inner: Input<M>,
}

impl<M: 'static> CommandInput<M> {
    fn new(placeholder: &str) -> Self {
        Self {
            inner: Input::new().with_placeholder(placeholder),
        }
    }

    fn set_value(&mut self, value: String) {
        self.inner.set_value(value);
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.inner.set_cursor(cursor);
    }

    fn value(&self) -> &str {
        self.inner.value()
    }

    fn set_focus(&mut self, focused: bool) {
        self.inner.set_focus(focused);
    }
}

impl<M: 'static> Widget<M> for CommandInput<M> {
    fn default_css(&self) -> &'static str {
        // From Python Textual: textual.command.CommandInput.DEFAULT_CSS
        // Note: background-tint: 0% converted to transparent 0% for parser compatibility
        r#"
    CommandInput, CommandInput:focus {
        border: blank;
        width: 1fr;
        padding-left: 0;
        background: transparent;
        background-tint: transparent 0%;
    }
    "#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "CommandInput";
        meta.type_names = vec!["CommandInput", "Input", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
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
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        self.inner.on_mouse_with_sender(event, region)
    }

    fn take_pending_action(&self) -> Option<String> {
        self.inner.take_pending_action()
    }

    fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
    }

    fn set_focus(&mut self, is_focused: bool) {
        self.inner.set_focus(is_focused);
    }

    fn is_focused(&self) -> bool {
        self.inner.is_focused()
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.inner.set_visible(visible);
    }

    fn is_loading(&self) -> bool {
        self.inner.is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.inner.set_loading(loading);
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled);
    }

    fn count_focusable(&self) -> usize {
        self.inner.count_focusable()
    }

    fn clear_focus(&mut self) {
        self.inner.clear_focus();
    }

    fn focus_nth(&mut self, n: usize) -> bool {
        self.inner.focus_nth(n)
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
        "CommandInput"
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

/// The command palette command list.
///
/// Python: `class CommandList(OptionList, can_focus=False)`
struct CommandList<M: 'static> {
    inner: OptionList<M>,
}

impl<M: 'static> CommandList<M> {
    fn new() -> Self {
        Self {
            inner: OptionList::new(Vec::new()).with_markup(true),
        }
    }

    fn set_items_with_state(&mut self, items: Vec<OptionItem>) {
        self.inner.set_items_with_state(items);
    }

    fn set_selected(&mut self, selected: usize) {
        self.inner.set_selected(selected);
    }

    fn take_pending_selection(&mut self) -> Option<usize> {
        self.inner.take_pending_selection()
    }
}

impl<M: 'static> Widget<M> for CommandList<M> {
    fn default_css(&self) -> &'static str {
        // From Python Textual: textual.command.CommandList.DEFAULT_CSS
        r#"
    CommandList {
        visibility: hidden;
        border-top: blank;
        border-bottom: hkey black;
        border-left: none;
        border-right: none;
        height: auto;
        max-height: 70vh;
        background: $surface;
        padding: 0;
    }

    CommandList:dark {
        background: $panel-darken-1;
    }

    CommandList:focus {
        border: blank;
    }

    CommandList.--visible {
        visibility: visible;
    }

    CommandList.--populating {
        border-bottom: none;
    }

    CommandList > .option-list--option-highlighted {
        color: $block-cursor-blurred-foreground;
        background: $block-cursor-blurred-background;
        text-style: $block-cursor-blurred-text-style;
    }

    CommandList:nocolor > .option-list--option-highlighted {
        text-style: reverse;
    }

    CommandList > .option-list--option {
        padding: 0 2;
        color: $foreground;
        text-style: bold;
    }
    "#
    }

    fn render(&self, canvas: &mut Canvas, region: Region) {
        self.inner.render(canvas, region);
    }

    fn desired_size(&self) -> Size {
        self.inner.desired_size()
    }

    fn get_meta(&self) -> WidgetMeta {
        let mut meta = self.inner.get_meta();
        meta.type_name = "CommandList";
        meta.type_names = vec!["CommandList", "OptionList", "Widget", "DOMNode"];
        meta
    }

    fn get_state(&self) -> WidgetStates {
        self.inner.get_state()
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
        self.inner.on_event(key)
    }

    fn on_mouse(&mut self, event: MouseEvent, region: Region) -> Option<M> {
        self.inner.on_mouse(event, region)
    }

    fn on_mouse_with_sender(
        &mut self,
        event: MouseEvent,
        region: Region,
    ) -> Option<(M, crate::widget::SenderInfo)> {
        self.inner.on_mouse_with_sender(event, region)
    }

    fn take_pending_action(&self) -> Option<String> {
        self.inner.take_pending_action()
    }

    fn is_focusable(&self) -> bool {
        // Python parity: CommandList can_focus=False
        false
    }

    fn set_focus(&mut self, _is_focused: bool) {
        // ignored
    }

    fn is_focused(&self) -> bool {
        false
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&mut self, visible: bool) {
        self.inner.set_visible(visible);
    }

    fn is_loading(&self) -> bool {
        self.inner.is_loading()
    }

    fn set_loading(&mut self, loading: bool) {
        self.inner.set_loading(loading);
    }

    fn is_disabled(&self) -> bool {
        self.inner.is_disabled()
    }

    fn set_disabled(&mut self, disabled: bool) {
        self.inner.set_disabled(disabled);
    }

    fn count_focusable(&self) -> usize {
        0
    }

    fn clear_focus(&mut self) {
        // no-op
    }

    fn focus_nth(&mut self, _n: usize) -> bool {
        false
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
        "CommandList"
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

#[derive(Debug, Clone)]
struct CommandEntry {
    display: String,
    action: Option<String>,
    text: String,
    help: Option<String>,
    score: f32,
    sequence: u64,
    disabled: bool,
}

impl CommandEntry {
    fn from_hit(hit: Hit, sequence: u64, help_style: &str) -> Self {
        let display = build_prompt(&hit.match_display, hit.help.as_deref(), help_style);
        Self {
            display,
            action: Some(hit.action),
            text: hit.text,
            help: hit.help,
            score: hit.score,
            sequence,
            disabled: false,
        }
    }

    fn from_discovery(hit: DiscoveryHit, sequence: u64, help_style: &str) -> Self {
        let display = build_prompt(&hit.display, hit.help.as_deref(), help_style);
        Self {
            display,
            action: Some(hit.action),
            text: hit.text,
            help: hit.help,
            score: 0.0,
            sequence,
            disabled: false,
        }
    }

    fn no_matches(help_style: &str) -> Self {
        let display = build_prompt("[dim]No matches found[/]", None, help_style);
        Self {
            display,
            action: None,
            text: "No matches found".to_string(),
            help: None,
            score: 0.0,
            sequence: 0,
            disabled: true,
        }
    }
}

fn build_prompt(prompt: &str, help: Option<&str>, help_style: &str) -> String {
    let mut lines = Vec::new();
    lines.push(prompt.to_string());
    if let Some(help) = help {
        if help_style.is_empty() {
            lines.push(escape_markup(help));
        } else {
            lines.push(format!("[{}]{}[/]", help_style, escape_markup(help)));
        }
    }
    lines.join("\n")
}

enum PaletteUpdate {
    ProviderResults {
        generation: u64,
        hits: Vec<CommandHit>,
    },
    ShowBusy {
        generation: u64,
    },
    ShowNoMatches {
        generation: u64,
        query_non_empty: bool,
    },
}

/// Command palette overlay widget.
pub struct CommandPalette<M: 'static> {
    inner: Container<M>,
    visible: bool,
    focused: bool,

    /// Parity with Python's `run_on_select` (if false, show a "go" button).
    run_on_select: bool,

    query: String,
    entries: Vec<CommandEntry>,
    pending_entries: Vec<CommandEntry>,
    selected: usize,
    list_visible: bool,
    show_busy: bool,
    clear_current: bool,
    sequence_counter: u64,
    expected_providers: usize,
    completed_providers: usize,
    providers: Vec<Arc<Mutex<Box<dyn Provider>>>>,
    update_tx: mpsc::UnboundedSender<PaletteUpdate>,
    update_rx: mpsc::UnboundedReceiver<PaletteUpdate>,
    search_generation: u64,
    search_handles: Vec<JoinHandle<()>>,
    busy_handle: Option<JoinHandle<()>>,
    no_matches_handle: Option<JoinHandle<()>>,
    last_update: Instant,
    match_style: String,
    help_style: String,

    pending_action: RefCell<Option<String>>,
    selected_action: Option<String>,
    pending_events: Vec<CommandPaletteEvent>,

    // Cached child indices (mirrors Python widget structure).
    container_index: usize,
    input_row_index: usize,
    input_index: usize,
    results_index: usize,
    list_index: usize,
    loading_index: usize,

    restore_focus: Option<usize>,
}

impl<M: 'static> CommandPalette<M> {
    pub fn new() -> Self {
        Self::new_with_run_on_select(true)
    }

    pub fn new_with_run_on_select(run_on_select: bool) -> Self {
        // Python hierarchy:
        // CommandPalette
        //   Vertical(id="--container")
        //     Horizontal(id="--input")
        //       SearchIcon
        //       CommandInput
        //       (optional Button)
        //     Vertical(id="--results")
        //       CommandList
        //       LoadingIndicator

        let search_icon: Box<dyn Widget<M>> = Box::new(SearchIcon::<M>::new());
        let input: Box<dyn Widget<M>> = Box::new(CommandInput::<M>::new("Search for commands…"));

        let input_children: Vec<Box<dyn Widget<M>>> = if run_on_select {
            vec![search_icon, input]
        } else {
            let go_button: Box<dyn Widget<M>> =
                Box::new(Button::new("\u{25b6}").with_action("app.command_palette.submit"));
            vec![search_icon, input, go_button]
        };

        let input_row: Box<dyn Widget<M>> = Box::new(
            Container::new(input_children)
                .with_id("--input")
                .with_layout(Layout::Horizontal),
        );

        let results: Box<dyn Widget<M>> = Box::new(
            Container::new(vec![
                Box::new(CommandList::<M>::new()) as Box<dyn Widget<M>>,
                Box::new(LoadingIndicator::new()) as Box<dyn Widget<M>>,
            ])
            .with_id("--results")
            .with_layout(Layout::Vertical),
        );

        let container: Box<dyn Widget<M>> = Box::new(
            Container::new(vec![input_row, results])
                .with_id("--container")
                .with_layout(Layout::Vertical),
        );

        let inner = Container::new(vec![container])
            .with_id("--command-palette")
            .with_classes("--textual-command-palette");

        let (update_tx, update_rx) = mpsc::unbounded_channel();

        let mut palette = Self {
            inner,
            visible: false,
            focused: false,
            run_on_select,
            query: String::new(),
            entries: Vec::new(),
            pending_entries: Vec::new(),
            selected: 0,
            list_visible: false,
            show_busy: false,
            clear_current: false,
            sequence_counter: 0,
            expected_providers: 0,
            completed_providers: 0,
            providers: Vec::new(),
            update_tx,
            update_rx,
            search_generation: 0,
            search_handles: Vec::new(),
            busy_handle: None,
            no_matches_handle: None,
            last_update: Instant::now(),
            match_style: "bold underline".to_string(),
            help_style: "dim".to_string(),
            pending_action: RefCell::new(None),
            selected_action: None,
            pending_events: Vec::new(),
            container_index: 0,
            input_row_index: 0,
            input_index: 1,
            results_index: 1,
            list_index: 0,
            loading_index: 1,
            restore_focus: None,
        };

        palette.refresh_results();
        palette
    }

    pub fn open(&mut self) {
        if !self.visible {
            self.visible = true;
            // Python shows the DOM once ready.
            self.inner.add_class("-ready");
            self.sync_visibility_classes();
            self.enqueue_event(CommandPaletteEvent::Opened);
            self.startup_providers();
            self.start_search(String::new(), Some(0));
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
        self.close_with_selected(false);
    }

    pub fn is_open(&self) -> bool {
        self.visible
    }

    pub fn set_commands(&mut self, commands: Vec<(String, String, Option<String>)>) {
        let simple_commands = commands
            .into_iter()
            .map(SimpleCommand::from)
            .collect::<Vec<_>>();
        self.set_providers(vec![Box::new(SimpleProvider::new(simple_commands))]);
    }

    pub fn set_providers(&mut self, providers: Vec<Box<dyn Provider>>) {
        self.cancel_search();
        self.providers = providers
            .into_iter()
            .map(|provider| Arc::new(Mutex::new(provider)))
            .collect();
        self.expected_providers = self.providers.len();
        if self.visible {
            self.startup_providers();
            self.start_search(self.query.clone(), None);
        }
    }

    pub fn submit(&mut self) {
        if self.list_visible {
            self.activate_selected();
        } else if let Some(action) = self.selected_action.take() {
            *self.pending_action.borrow_mut() = Some(action);
            self.close_with_selected(true);
        }
    }

    pub fn take_restore_focus(&mut self) -> Option<usize> {
        if self.visible {
            None
        } else {
            self.restore_focus.take()
        }
    }

    pub fn take_events(&mut self) -> Vec<CommandPaletteEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn drain_updates(&mut self) -> bool {
        let mut changed = false;
        while let Ok(update) = self.update_rx.try_recv() {
            if self.apply_update(update) {
                changed = true;
            }
        }
        if changed {
            self.inner.mark_dirty();
        }
        changed
    }

    pub fn tick(&mut self) -> bool {
        if self.show_busy {
            if let Some(loading) = self.loading_mut() {
                loading.tick();
                self.inner.mark_dirty();
                return true;
            }
        }
        false
    }

    fn enqueue_event(&mut self, event: CommandPaletteEvent) {
        self.pending_events.push(event);
    }

    fn close_with_selected(&mut self, option_selected: bool) {
        if self.visible {
            self.visible = false;
            self.cancel_search();
            self.query.clear();
            self.selected = 0;
            self.selected_action = None;
            self.entries.clear();
            self.pending_entries.clear();
            self.list_visible = false;
            self.show_busy = false;
            self.sync_visibility_classes();
            self.refresh_results();
            self.enqueue_event(CommandPaletteEvent::Closed { option_selected });
            self.shutdown_providers();
            self.inner.mark_dirty();
        }
    }

    fn cancel_search(&mut self) {
        for handle in self.search_handles.drain(..) {
            handle.abort();
        }
        if let Some(handle) = self.busy_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.no_matches_handle.take() {
            handle.abort();
        }
        self.show_busy = false;
        if let Some(loading) = self.loading_mut() {
            loading.set_visible(false);
            loading.remove_class("--visible");
        }
        if let Some(list) = self.list_mut() {
            list.remove_class("--populating");
        }
        self.expected_providers = 0;
        self.completed_providers = 0;
    }

    fn startup_providers(&mut self) {
        for provider in &self.providers {
            let provider = Arc::clone(provider);
            let match_style = self.match_style.clone();
            tokio::spawn(async move {
                let mut provider = provider.lock().await;
                provider.set_match_style(match_style);
                provider.startup().await;
            });
        }
    }

    fn shutdown_providers(&mut self) {
        for provider in &self.providers {
            let provider = Arc::clone(provider);
            tokio::spawn(async move {
                let mut provider = provider.lock().await;
                provider.shutdown().await;
            });
        }
    }

    fn start_search(&mut self, query: String, cursor: Option<usize>) {
        self.cancel_search();
        self.query = query;
        self.selected = 0;
        self.selected_action = None;
        self.pending_entries.clear();
        self.sequence_counter = 0;
        self.completed_providers = 0;
        self.expected_providers = self.providers.len();
        self.clear_current = true;
        self.last_update = Instant::now();
        self.search_generation = self.search_generation.wrapping_add(1);
        let generation = self.search_generation;
        let search_value = self.query.trim().to_string();
        if self.visible && self.selected_action.is_none() {
            self.list_visible = true;
        }
        self.update_input_widget(cursor);
        self.sync_visibility_classes();

        if self.expected_providers == 0 {
            self.entries.clear();
            self.list_visible = false;
            self.sync_visibility_classes();
            self.refresh_results();
            if !search_value.is_empty() {
                self.spawn_no_matches_timer(generation, true);
            }
            return;
        }

        self.spawn_busy_timer(generation);

        let query = search_value.clone();
        let match_style = self.match_style.clone();
        let providers = self.providers.clone();

        for provider in providers {
            let provider = Arc::clone(&provider);
            let query = query.clone();
            let tx = self.update_tx.clone();
            let match_style = match_style.clone();
            let generation = generation;
            let handle = tokio::spawn(async move {
                let mut provider = provider.lock().await;
                provider.set_match_style(match_style);
                let hits: Vec<CommandHit> = if query.is_empty() {
                    provider
                        .discover()
                        .await
                        .into_iter()
                        .map(CommandHit::Discovery)
                        .collect()
                } else {
                    provider
                        .search(&query)
                        .await
                        .into_iter()
                        .map(CommandHit::Search)
                        .collect()
                };
                let _ = tx.send(PaletteUpdate::ProviderResults { generation, hits });
            });
            self.search_handles.push(handle);
        }
    }

    fn spawn_busy_timer(&mut self, generation: u64) {
        let tx = self.update_tx.clone();
        self.busy_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = tx.send(PaletteUpdate::ShowBusy { generation });
        }));
    }

    fn spawn_no_matches_timer(&mut self, generation: u64, query_non_empty: bool) {
        let tx = self.update_tx.clone();
        self.no_matches_handle = Some(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let _ = tx.send(PaletteUpdate::ShowNoMatches {
                generation,
                query_non_empty,
            });
        }));
    }

    fn apply_update(&mut self, update: PaletteUpdate) -> bool {
        match update {
            PaletteUpdate::ProviderResults { generation, hits } => {
                if generation != self.search_generation {
                    return false;
                }
                self.completed_providers += 1;

                for hit in hits {
                    let entry = match hit {
                        CommandHit::Search(hit) => {
                            self.sequence_counter += 1;
                            CommandEntry::from_hit(hit, self.sequence_counter, &self.help_style)
                        }
                        CommandHit::Discovery(hit) => {
                            self.sequence_counter += 1;
                            CommandEntry::from_discovery(
                                hit,
                                self.sequence_counter,
                                &self.help_style,
                            )
                        }
                    };
                    self.pending_entries.push(entry);
                }

                let now = Instant::now();
                if now.duration_since(self.last_update) > Duration::from_millis(250) {
                    self.flush_pending_entries();
                    self.last_update = now;
                }

                if self.completed_providers >= self.expected_providers {
                    if let Some(handle) = self.busy_handle.take() {
                        handle.abort();
                    }
                    self.show_busy = false;
                    let mut cleared = false;
                    if self.clear_current && self.pending_entries.is_empty() {
                        self.entries.clear();
                        self.clear_current = false;
                        cleared = true;
                    }
                    self.flush_pending_entries();
                    if cleared {
                        self.refresh_results();
                    }

                    if self.entries.is_empty() && !self.query.trim().is_empty() {
                        self.spawn_no_matches_timer(generation, true);
                    }
                }
                true
            }
            PaletteUpdate::ShowBusy { generation } => {
                if generation != self.search_generation {
                    return false;
                }
                if self.list_visible {
                    self.show_busy = true;
                    if let Some(loading) = self.loading_mut() {
                        loading.set_visible(true);
                        loading.add_class("--visible");
                    }
                    if let Some(list) = self.list_mut() {
                        list.add_class("--populating");
                    }
                }
                true
            }
            PaletteUpdate::ShowNoMatches {
                generation,
                query_non_empty,
            } => {
                if generation != self.search_generation {
                    return false;
                }
                if query_non_empty {
                    self.entries = vec![CommandEntry::no_matches(&self.help_style)];
                    self.selected = 0;
                    self.list_visible = true;
                    self.sync_visibility_classes();
                    self.update_list_widget();
                } else {
                    self.list_visible = false;
                    self.sync_visibility_classes();
                }
                true
            }
        }
    }

    fn flush_pending_entries(&mut self) {
        if self.pending_entries.is_empty() {
            return;
        }
        if self.clear_current {
            self.entries.clear();
            self.clear_current = false;
        }
        self.entries.append(&mut self.pending_entries);
        self.refresh_results();
    }

    fn container_mut(&mut self) -> Option<&mut Container<M>> {
        self.inner
            .get_child_mut(self.container_index)
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Container<M>>())
    }

    fn input_row_mut(&mut self) -> Option<&mut Container<M>> {
        let index = self.input_row_index;
        self.container_mut()
            .and_then(|container| container.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Container<M>>())
    }

    fn results_mut(&mut self) -> Option<&mut Container<M>> {
        let index = self.results_index;
        self.container_mut()
            .and_then(|container| container.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<Container<M>>())
    }

    fn input_mut(&mut self) -> Option<&mut CommandInput<M>> {
        let index = self.input_index;
        self.input_row_mut()
            .and_then(|row| row.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<CommandInput<M>>())
    }

    fn list_mut(&mut self) -> Option<&mut CommandList<M>> {
        let index = self.list_index;
        self.results_mut()
            .and_then(|results| results.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<CommandList<M>>())
    }

    fn loading_mut(&mut self) -> Option<&mut LoadingIndicator<M>> {
        let index = self.loading_index;
        self.results_mut()
            .and_then(|results| results.get_child_mut(index))
            .and_then(|child| child.as_any_mut())
            .and_then(|child| child.downcast_mut::<LoadingIndicator<M>>())
    }

    fn refresh_results(&mut self) {
        self.entries.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.sequence.cmp(&b.sequence))
        });

        if self.entries.is_empty() {
            self.selected = 0;
            self.list_visible = false;
        } else {
            self.selected = self.selected.min(self.entries.len() - 1);
            self.normalize_selection();
            self.list_visible = self.selected_action.is_none();
        }

        self.update_list_widget();
        self.update_input_widget(None);
        self.sync_visibility_classes();
        if self.list_visible {
            self.emit_highlight();
        }

        let show_busy = self.show_busy;
        if let Some(loading) = self.loading_mut() {
            if show_busy {
                loading.set_visible(true);
                loading.add_class("--visible");
            } else {
                loading.set_visible(false);
                loading.remove_class("--visible");
            }
        }

        if let Some(list) = self.list_mut() {
            if show_busy {
                list.add_class("--populating");
            } else {
                list.remove_class("--populating");
            }
        }
    }

    fn sync_visibility_classes(&mut self) {
        let list_visible = self.visible && self.list_visible;

        if let Some(list) = self.list_mut() {
            if list_visible {
                list.add_class("--visible");
            } else {
                list.remove_class("--visible");
            }
        }

        if let Some(input_row) = self.input_row_mut() {
            if list_visible {
                input_row.add_class("--list-visible");
            } else {
                input_row.remove_class("--list-visible");
            }
        }

        if !list_visible {
            self.show_busy = false;
            if let Some(loading) = self.loading_mut() {
                loading.set_visible(false);
                loading.remove_class("--visible");
            }
            if let Some(list) = self.list_mut() {
                list.remove_class("--populating");
            }
        }
    }

    fn update_list_widget(&mut self) {
        let selected = self.selected;

        let items: Vec<OptionItem> = self
            .entries
            .iter()
            .map(|entry| OptionItem::new(entry.display.clone()).disabled(entry.disabled))
            .collect();

        if let Some(list) = self.list_mut() {
            list.set_items_with_state(items);
            list.set_selected(selected);
        }
    }

    fn update_input_widget(&mut self, cursor: Option<usize>) {
        let query = self.query.clone();
        if let Some(input) = self.input_mut() {
            if input.value() != query {
                input.set_value(query);
            }
            if let Some(cursor) = cursor {
                input.set_cursor(cursor);
            }
        }
    }

    fn emit_highlight(&mut self) {
        if !self.visible || !self.list_visible {
            return;
        }
        let Some(entry) = self.entries.get(self.selected) else {
            return;
        };
        if entry.disabled {
            return;
        }
        let event = CommandPaletteEvent::OptionHighlighted(CommandPaletteHighlight {
            index: self.selected,
            text: entry.text.clone(),
            action: entry.action.clone().unwrap_or_default(),
            help: entry.help.clone(),
        });
        self.enqueue_event(event);
    }

    fn normalize_selection(&mut self) {
        if self.entries.is_empty() {
            self.selected = 0;
            return;
        }
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len() - 1;
        }
        if self
            .entries
            .get(self.selected)
            .map_or(false, |entry| entry.disabled)
        {
            if let Some(index) = self.next_enabled_index(self.selected, 1) {
                self.selected = index;
                return;
            }
            if let Some(index) = self.next_enabled_index(self.selected, -1) {
                self.selected = index;
            }
        }
    }

    fn next_enabled_index(&self, start: usize, delta: i32) -> Option<usize> {
        let len = self.entries.len() as i32;
        let mut index = start as i32;
        loop {
            index += delta;
            if index < 0 || index >= len {
                return None;
            }
            let idx = index as usize;
            if !self.entries.get(idx).map_or(true, |entry| entry.disabled) {
                return Some(idx);
            }
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let len = self.entries.len() as i32;
        let mut index = self.selected as i32;
        loop {
            index += delta;
            if index < 0 || index >= len {
                return;
            }
            let next = index as usize;
            if !self.entries[next].disabled {
                self.selected = next;
                self.update_list_widget();
                self.emit_highlight();
                return;
            }
        }
    }

    fn activate_selected(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let Some(entry) = self.entries.get(self.selected).cloned() else {
            return;
        };
        if entry.disabled {
            return;
        }

        if self.run_on_select {
            if let Some(action) = entry.action.clone() {
                *self.pending_action.borrow_mut() = Some(action);
                self.close_with_selected(true);
            }
        } else {
            self.query = entry.text.clone();
            self.selected_action = entry.action;
            self.list_visible = false;
            self.entries.clear();
            self.pending_entries.clear();
            self.selected = 0;
            self.sync_visibility_classes();
            let cursor = grapheme_count(&self.query);
            self.update_input_widget(Some(cursor));
            self.refresh_results();
        }
    }

    fn handle_list_mouse_selection(&mut self, event: MouseEvent) {
        let Some(list) = self.list_mut() else {
            return;
        };
        let Some(index) = list.take_pending_selection() else {
            return;
        };

        if index != self.selected {
            self.selected = index;
            self.update_list_widget();
        }
        self.emit_highlight();

        if matches!(
            event.kind,
            MouseEventKind::Up(crossterm::event::MouseButton::Left)
        ) {
            self.activate_selected();
        }
    }
}

impl<M: 'static> Default for CommandPalette<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: 'static> Widget<M> for CommandPalette<M> {
    fn default_css(&self) -> &'static str {
        // From Python Textual: textual.command.CommandPalette.DEFAULT_CSS
        // Note: Nested selectors flattened for TCSS parser compatibility
        r#"
CommandPalette:inline {
    min-height: 20;
}

CommandPalette {
    color: $foreground;
    background: $background 60%;
    align-horizontal: center;
}

CommandPalette #--container {
    display: none;
}

CommandPalette:ansi {
    background: transparent;
}

CommandPalette.-ready #--container {
    display: block;
}

CommandPalette > .command-palette--help-text {
    color: $text-muted;
    background: transparent;
    text-style: none;
}

CommandPalette > .command-palette--highlight {
    text-style: bold underline;
}

CommandPalette:nocolor > .command-palette--highlight {
    text-style: underline;
}

CommandPalette > Vertical {
    margin-top: 3;
    height: 100%;
    visibility: hidden;
    background: $surface;
}

CommandPalette > Vertical:dark {
    background: $panel-darken-1;
}

CommandPalette #--input {
    height: auto;
    visibility: visible;
    border: hkey black 50%;
}

CommandPalette #--input.--list-visible {
    border-bottom: none;
}

CommandPalette #--input Label {
    margin-top: 1;
    margin-left: 1;
}

CommandPalette #--input Button {
    min-width: 7;
    margin-right: 1;
}

CommandPalette #--results {
    overlay: screen;
    height: auto;
}

CommandPalette LoadingIndicator {
    height: auto;
    visibility: hidden;
    border-bottom: hkey $border;
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
            KeyCode::Esc => self.close_with_selected(false),
            KeyCode::Enter => self.submit(),
            KeyCode::Up => {
                if !self.list_visible && !self.entries.is_empty() {
                    self.list_visible = true;
                    self.sync_visibility_classes();
                    self.update_list_widget();
                    self.emit_highlight();
                } else {
                    self.move_selection(-1);
                }
            }
            KeyCode::Down => {
                if !self.list_visible && !self.entries.is_empty() {
                    self.list_visible = true;
                    self.sync_visibility_classes();
                    self.update_list_widget();
                    self.emit_highlight();
                } else {
                    self.move_selection(1);
                }
            }
            _ => {
                if let Some(input) = self.input_mut() {
                    let before = input.value().to_string();
                    input.on_event(key);
                    let after = input.value().to_string();
                    if after != before {
                        self.query = after;
                        self.selected = 0;
                        self.selected_action = None;
                        self.start_search(self.query.clone(), None);
                    }
                }
            }
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
            self.close_with_selected(false);
            return None;
        }

        let result = self.inner.on_mouse(event, region);
        self.handle_list_mouse_selection(event);
        result
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
            self.close_with_selected(false);
            return None;
        }

        let result = self.inner.on_mouse_with_sender(event, region);
        self.handle_list_mouse_selection(event);
        result
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
