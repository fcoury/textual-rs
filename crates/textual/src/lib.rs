pub mod border_box;
pub mod border_chars;
pub mod border_render;
pub mod box_drawing;
pub mod canvas;
pub mod containers;
pub mod content;
pub mod context;
pub mod error;
pub mod fraction;
pub mod keyline_canvas;
pub mod layouts;
mod log_init;
mod macros;
pub mod message;
pub mod render_cache;
pub mod scroll;
pub mod scrollbar;
pub mod segment;
pub mod strip;
pub mod style_resolver;
pub mod svg;
pub mod testing;
pub mod tree;
pub mod visual;
pub mod widget;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
pub use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::{cursor, execute, terminal};
use futures::StreamExt;
use std::collections::{HashSet, VecDeque};
use tokio::sync::mpsc;

pub use canvas::{Canvas, Region, Size};
pub use containers::{
    Center, Middle, container::Container, grid::Grid, horizontal::Horizontal,
    horizontal_scroll::HorizontalScroll, item_grid::ItemGrid, scrollable::ScrollableContainer,
    vertical::Vertical, vertical_scroll::VerticalScroll,
};
pub use context::{AppContext, IntervalHandle, MountContext};
pub use error::Result;
pub use fraction::Fraction;
pub use log_init::init_logger;
pub use message::MessageEnvelope;
pub use scroll::{ScrollMessage, ScrollState};
pub use scrollbar::{ScrollBarRender, ScrollbarGlyphs};
pub use tcss::TcssError;
pub use tree::{DOMQuery, clear_all_hover, collect_pending_actions_mut};
pub use visual::VisualType;
pub use widget::label::{Label, LabelVariant};
pub use widget::static_widget::Static;

// Re-export the log crate so users can use textual::log::info!, etc.
pub use log;
pub use tcss::{parser::parse_stylesheet, types::Theme};
pub use textual_macros::{ui, widget};
pub use widget::{
    Compose, Widget, placeholder::Placeholder, placeholder::PlaceholderVariant,
    placeholder::reset_placeholder_counter, ruler::Ruler, ruler::RulerOrientation,
    screen::Breakpoint, screen::Screen, scrollbar::ScrollBar, scrollbar_corner::ScrollBarCorner,
    switch::Switch,
};

/// Helper for building widget vectors from iterators.
///
/// Provides a more ergonomic way to build dynamic widget lists.
///
/// # Example
///
/// ```ignore
/// let items = vec!["a", "b", "c"];
/// let widgets = widgets_from_iter(items, |item| {
///     Box::new(Static::new(item)) as Box<dyn Widget<_>>
/// });
/// ```
pub fn widgets_from_iter<M, T, F>(
    iter: impl IntoIterator<Item = T>,
    f: F,
) -> Vec<Box<dyn Widget<M>>>
where
    F: Fn(T) -> Box<dyn Widget<M>>,
{
    iter.into_iter().map(f).collect()
}

use crate::{
    error::TextualError,
    style_resolver::{InheritedContext, resolve_dirty_styles, resolve_styles},
    tree::WidgetTree,
};

/// Collect default CSS from a widget and all its descendants.
///
/// Walks the widget tree and collects unique `default_css()` strings from all widgets.
/// These are prepended to the app's CSS to ensure widget defaults are applied with
/// lower specificity than app-level styles.
fn collect_default_css<M>(widget: &mut dyn Widget<M>, collected: &mut HashSet<&'static str>) {
    let default_css = widget.default_css();
    if !default_css.is_empty() {
        collected.insert(default_css);
    }
    widget.for_each_child(&mut |child| {
        collect_default_css(child, collected);
    });
}

/// Build a combined stylesheet from widget defaults and app CSS.
///
/// Widget default CSS is prepended (lower specificity), so app CSS can override.
fn build_combined_css<M>(root: &mut dyn Widget<M>, app_css: &str) -> String {
    let mut defaults: HashSet<&'static str> = HashSet::new();
    collect_default_css(root, &mut defaults);

    // Concatenate defaults (order doesn't matter, app CSS will override)
    let mut combined = String::new();

    // First add base widget CSS (universal defaults like scrollbar and link styling)
    combined.push_str(widget::screen::Screen::<M>::base_widget_css());
    combined.push('\n');

    for css in defaults {
        combined.push_str(css);
        combined.push('\n');
    }
    combined.push_str(app_css);
    combined
}

/// The main application trait. Implement this to create a TUI application.
///
/// The `Message` associated type (from `Compose`) defines the events your UI can produce.
/// This enables type-safe event handling with exhaustive pattern matching.
pub trait App: Compose
where
    Self::Message: Send + 'static,
{
    const CSS: &'static str = "";

    /// Handle a message produced by a widget.
    ///
    /// Messages are wrapped in an envelope that carries metadata:
    /// - `envelope.message` - the actual message payload
    /// - `envelope.sender_id` - optional widget ID that produced the message
    /// - `envelope.sender_type` - type name of the widget that produced the message
    ///
    /// Use pattern matching on `envelope.message` to handle each variant of your Message enum.
    fn handle_message(&mut self, _envelope: MessageEnvelope<Self::Message>) {}

    /// Handle global key events (e.g., 'q' to quit).
    /// Called after widget event handling.
    fn on_key(&mut self, key: KeyCode);

    /// Return true when the application should exit.
    fn should_quit(&self) -> bool;

    /// Handle an action string from a link click (e.g., "app.bell", "app.quit").
    ///
    /// Override this to handle custom actions. Return `true` if the action was handled.
    /// The default implementation handles built-in actions:
    /// - `app.quit` or `quit` - sets a flag to quit the app
    /// - `app.bell` or `bell` - triggers the terminal bell
    ///
    /// # Example
    /// ```ignore
    /// fn on_action(&mut self, action: &str) -> bool {
    ///     match action {
    ///         "my_custom_action" => {
    ///             // Handle custom action
    ///             true
    ///         }
    ///         _ => false, // Let default handling proceed
    ///     }
    /// }
    /// ```
    fn on_action(&mut self, _action: &str) -> bool {
        false // Default: no custom handling
    }

    /// Request the application to quit.
    ///
    /// This is called by the "app.quit" action from link clicks.
    /// Override this to set a quit flag in your app state.
    ///
    /// The default implementation does nothing - you must implement this
    /// if you want `app.quit` links to work.
    fn request_quit(&mut self) {
        // Default: do nothing. Apps must override this.
    }

    /// Trigger the terminal bell sound.
    ///
    /// This is called by the "app.bell" action from link clicks.
    /// The default implementation writes the bell character to stdout.
    fn bell(&mut self) {
        use std::io::Write;
        let _ = std::io::stdout().write_all(b"\x07");
        let _ = std::io::stdout().flush();
    }

    /// Dispatch an action string from a link click.
    ///
    /// This is called automatically by the event loop when a link with
    /// `[@click=action]` markup is clicked. It first calls `on_action()` to
    /// allow custom handling, then falls back to built-in actions.
    ///
    /// Built-in actions:
    /// - `app.quit` / `quit` - calls `request_quit()`
    /// - `app.bell` / `bell` - calls `bell()`
    fn dispatch_action(&mut self, action: &str) {
        // First, let the app handle custom actions
        if self.on_action(action) {
            return;
        }

        // Handle built-in actions
        match action {
            "app.quit" | "quit" => {
                self.request_quit();
            }
            "app.bell" | "bell" => {
                self.bell();
            }
            _ => {
                // Unknown action - log and ignore
                log::debug!("Unknown action: {}", action);
            }
        }
    }

    /// Returns the current focus index for the widget tree.
    /// The run loop uses this to set focus on the nth focusable widget.
    fn focus_index(&self) -> usize {
        0
    }

    /// Called once when the application starts, after the widget tree is built.
    ///
    /// Use this to start timers, spawn background tasks, query widgets, or
    /// perform other initialization. The `MountContext` provides access to
    /// both the widget tree and async messaging capabilities.
    ///
    /// # Example
    /// ```ignore
    /// fn on_mount(&mut self, ctx: &mut MountContext<Self::Message>) {
    ///     // Query and modify widgets
    ///     ctx.with_widget_by_id("my-label", |widget| {
    ///         widget.set_border_title("Textual Rocks!");
    ///     });
    ///
    ///     // Set up timers
    ///     ctx.set_interval(Duration::from_secs(1), || Message::Tick);
    /// }
    /// ```
    fn on_mount(&mut self, _ctx: &mut MountContext<Self::Message>) {
        // Default: do nothing
    }

    /// Return true to rebuild the widget tree via `compose()`.
    ///
    /// **Default behavior (false)**: Persistent tree model - widgets own their state.
    /// The tree is built once and widgets update themselves. Use this when widgets
    /// like Switch manage their own toggle state.
    ///
    /// **Override to return true**: Elm-style model - app owns all state.
    /// The tree is rebuilt after every message, reading state from the App.
    /// Use this when App fields drive widget state (e.g., loading indicators).
    ///
    /// # Example
    /// ```ignore
    /// fn needs_recompose(&self) -> bool {
    ///     true // Elm-style: rebuild tree after every state change
    /// }
    /// ```
    fn needs_recompose(&self) -> bool {
        false
    }

    /// Returns custom horizontal breakpoints for responsive layouts.
    ///
    /// Breakpoints are (threshold, class_name) pairs. The class is applied
    /// when width >= threshold. The last matching breakpoint wins.
    ///
    /// Default: `[(0, "-narrow"), (80, "-wide")]`
    ///
    /// # Example
    /// ```ignore
    /// fn horizontal_breakpoints(&self) -> &'static [(u16, &'static str)] {
    ///     &[(0, "-narrow"), (40, "-normal"), (80, "-wide"), (120, "-very-wide")]
    /// }
    /// ```
    fn horizontal_breakpoints(&self) -> &'static [(u16, &'static str)] {
        widget::screen::DEFAULT_HORIZONTAL_BREAKPOINTS
    }

    /// Returns custom vertical breakpoints for responsive layouts.
    ///
    /// Breakpoints are (threshold, class_name) pairs. The class is applied
    /// when height >= threshold. The last matching breakpoint wins.
    ///
    /// Default: `[(0, "-short"), (24, "-tall")]`
    fn vertical_breakpoints(&self) -> &'static [(u16, &'static str)] {
        widget::screen::DEFAULT_VERTICAL_BREAKPOINTS
    }

    /// Run the application event loop.
    ///
    /// This uses a **persistent widget tree** - the tree is built once at startup
    /// and mutated in place. Events go to the existing widgets, which update their
    /// own state and mark themselves dirty for restyling.
    ///
    /// The event loop uses `tokio::select!` to poll:
    /// - Terminal events via `crossterm::event::EventStream`
    /// - Async messages via `tokio::sync::mpsc` channel
    ///
    /// Runtime handling:
    /// - No runtime: creates a new multi-threaded runtime
    /// - Multi-thread runtime: reuses via `block_in_place`
    /// - Current-thread runtime: returns an error (use `run_async` instead)
    fn run(&mut self) -> Result<()> {
        // Check if we're already inside a Tokio runtime
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                use tokio::runtime::RuntimeFlavor;
                match handle.runtime_flavor() {
                    RuntimeFlavor::MultiThread => {
                        // Multi-thread runtime - safe to use block_in_place
                        tokio::task::block_in_place(|| handle.block_on(self.run_inner()))
                    }
                    RuntimeFlavor::CurrentThread | _ => {
                        // Current-thread runtime - can't block without deadlock
                        Err(TextualError::RuntimeInit(
                            "Cannot call run() from a current-thread Tokio runtime. \
                             Use run_async().await instead, or use #[tokio::main] \
                             (multi-threaded by default)."
                                .to_string(),
                        ))
                    }
                }
            }
            Err(_) => {
                // No runtime - create a new one
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| TextualError::RuntimeInit(e.to_string()))?;
                rt.block_on(self.run_inner())
            }
        }
    }

    /// Run the application event loop asynchronously.
    ///
    /// Use this when you're already inside an async context and want to
    /// run the app without blocking. This is the preferred method when
    /// calling from a current-thread Tokio runtime.
    ///
    /// # Example
    /// ```ignore
    /// #[tokio::main(flavor = "current_thread")]
    /// async fn main() {
    ///     let mut app = MyApp::new();
    ///     app.run_async().await.unwrap();
    /// }
    /// ```
    fn run_async(&mut self) -> impl std::future::Future<Output = Result<()>> + '_ {
        self.run_inner()
    }

    /// Inner async run logic, separated for runtime flexibility.
    fn run_inner(&mut self) -> impl std::future::Future<Output = Result<()>> + '_ {
        async move {
            let mut stdout = std::io::stdout();
            // Enable raw mode, mouse capture, and enter alternate screen
            terminal::enable_raw_mode()?;
            execute!(
                stdout,
                terminal::EnterAlternateScreen,
                cursor::Hide,
                EnableMouseCapture
            )?;

            let result = self.event_loop_async().await;

            // Cleanup: Restore terminal state on exit
            execute!(
                stdout,
                DisableMouseCapture,
                cursor::Show,
                terminal::LeaveAlternateScreen
            )?;
            terminal::disable_raw_mode()?;

            result
        }
    }

    /// The main async event loop.
    fn event_loop_async(&mut self) -> impl std::future::Future<Output = Result<()>> + '_ {
        async move {
            // 1. Initial Setup: Build widget tree first, then collect default CSS
            let themes = tcss::types::Theme::standard_themes();
            let theme = themes
                .get("textual-dark")
                .cloned()
                .unwrap_or_else(|| tcss::types::Theme::new("default", true));

            let (mut cols, mut rows) = terminal::size()?;
            let mut canvas = Canvas::new(cols, rows);

            // 2. Build the widget tree ONCE (persistent tree)
            // Use WidgetTree for O(d) focus-targeted dispatch and message bubbling
            // DOM hierarchy: App > Screen > user widgets (matches Python Textual)
            let screen = Screen::new(self.compose())
                .with_horizontal_breakpoints(self.horizontal_breakpoints())
                .with_vertical_breakpoints(self.vertical_breakpoints());
            let root = Box::new(widget::app_widget::AppWidget::new(Box::new(screen)));
            let mut tree = WidgetTree::new(root);

            // Initialize Screen with current terminal size for breakpoints
            tree.root_mut().on_resize(Size::new(cols, rows));

            // 3. Collect widget default CSS and combine with app CSS
            // Widget defaults are prepended (lower specificity), app CSS overrides
            let combined_css = build_combined_css(tree.root_mut(), Self::CSS);
            let stylesheet = tcss::parser::parse_stylesheet(&combined_css)
                .map_err(|e| TextualError::InvalidCss(e.to_string()))?;

            // Set initial focus and cache the focus path
            tree.root_mut().clear_focus();
            tree.root_mut().focus_nth(self.focus_index());
            tree.update_focus(self.focus_index());

            // Initial style resolution for all widgets
            let mut ancestors = VecDeque::new();
            resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

            // 3. Create message channel for async communication
            let (tx, mut rx) = mpsc::unbounded_channel::<MessageEnvelope<Self::Message>>();

            // Call lifecycle hook with MountContext (provides widget tree access)
            // MountContext takes ownership of an AppContext, so we create one specifically for it
            let mount_app_ctx = AppContext::new(tx.clone());
            let mut mount_ctx = MountContext::new(mount_app_ctx, &mut tree);
            self.on_mount(&mut mount_ctx);

            // 4. Create async event stream
            let mut event_stream = EventStream::new();

            // Track the previous focus index to detect changes
            let mut last_focus_index = self.focus_index();

            // Flag to prevent redundant re-renders
            let mut needs_render = true;
            // Flag to trigger tree rebuild after state changes
            let mut needs_recompose = false;

            while !self.should_quit() {
                // Rebuild widget tree if app state changed
                if needs_recompose {
                    // DOM hierarchy: App > Screen > user widgets (matches Python Textual)
                    let screen = Screen::new(self.compose())
                        .with_horizontal_breakpoints(self.horizontal_breakpoints())
                        .with_vertical_breakpoints(self.vertical_breakpoints());
                    let root = Box::new(widget::app_widget::AppWidget::new(Box::new(screen)));
                    tree = WidgetTree::new(root);

                    // Re-apply resize to new tree so breakpoints are correct
                    tree.root_mut().on_resize(Size::new(cols, rows));

                    tree.root_mut().clear_focus();
                    tree.root_mut().focus_nth(self.focus_index());
                    tree.update_focus(self.focus_index());
                    last_focus_index = self.focus_index();

                    // Full style resolution for new tree
                    let mut ancestors = VecDeque::new();
                    resolve_styles(tree.root_mut(), &stylesheet, &theme, &mut ancestors);

                    needs_recompose = false;
                    needs_render = true;
                }

                // Check if focus changed
                let current_focus = self.focus_index();
                if current_focus != last_focus_index {
                    tree.root_mut().clear_focus();
                    tree.root_mut().focus_nth(current_focus);
                    tree.update_focus(current_focus);
                    last_focus_index = current_focus;
                    needs_render = true;
                }

                if needs_render {
                    // Resolve styles only for dirty widgets
                    let mut ancestors = VecDeque::new();
                    resolve_dirty_styles(
                        tree.root_mut(),
                        &stylesheet,
                        &theme,
                        &mut ancestors,
                        false,
                        &InheritedContext::from_theme(&theme),
                    );

                    canvas.clear();
                    let region = Region::from_u16(0, 0, cols, rows);
                    tree.root().render(&mut canvas, region);
                    canvas.flush()?;

                    needs_render = false;
                }

                // 5. Event Handling: Use tokio::select! for async polling
                // NOTE: Don't use `biased` here - it would starve the message channel
                // if the event stream keeps returning ready (mouse events, etc.)
                tokio::select! {

                    // Terminal events from crossterm
                    maybe_event = event_stream.next() => {
                        match maybe_event {
                            Some(Ok(Event::Key(key_event))) => {
                                // Dispatch key event to the focused widget using cached focus path
                                if let Some(msg) = tree.dispatch_key(key_event.code) {
                                    // Get sender info using cached focus path (O(d) access)
                                    let sender = tree.focused_sender_info();

                                    // Create envelope and bubble through ancestor widgets
                                    let envelope = MessageEnvelope::new(msg, sender.id.as_deref(), sender.type_name);
                                    let bubbled = tree.bubble_message(envelope);

                                    // App is always the final handler (even if bubbling was stopped)
                                    self.handle_message(bubbled);
                                    // Check if app wants tree rebuild (Elm-style)
                                    needs_recompose = self.needs_recompose();
                                }
                                self.on_key(key_event.code);
                                needs_render = true;
                            }
                            Some(Ok(Event::Resize(nw, nh))) => {
                                // Handle terminal window resizing
                                cols = nw;
                                rows = nh;
                                canvas = Canvas::new(cols, rows);

                                // Propagate resize to Screen for breakpoint updates
                                tree.root_mut().on_resize(Size::new(cols, rows));

                                needs_render = true;
                            }
                            Some(Ok(Event::Mouse(mouse_event))) => {
                                // Compute the full-screen region for mouse event routing
                                let region = Region::from_u16(0, 0, cols, rows);

                                // Clear hover state on all widgets before processing move events
                                // This ensures widgets that are no longer hovered clear their state
                                if matches!(mouse_event.kind, crossterm::event::MouseEventKind::Moved) {
                                    clear_all_hover(tree.root_mut());
                                }

                                // Route mouse event through widget tree hit-testing
                                // NOTE: Mouse events use hit-testing, not focus path.
                                // Full mouse bubbling would require on_mouse to track the hit path.
                                // For now, messages go directly to App without parent interception.
                                if let Some(msg) = tree.root_mut().on_mouse(mouse_event, region) {
                                    let envelope = MessageEnvelope::new(msg, None, "Widget");
                                    self.handle_message(envelope);
                                    // Check if app wants tree rebuild (Elm-style)
                                    needs_recompose = self.needs_recompose();
                                }

                                // Collect and dispatch pending actions from link clicks
                                let actions = collect_pending_actions_mut(tree.root_mut());
                                for action in actions {
                                    self.dispatch_action(&action);
                                }

                                needs_render = true;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => return Err(TextualError::IO(e)),
                            None => break, // Stream ended
                        }
                    }

                    // Messages from async tasks (timers, background work)
                    Some(envelope) = rx.recv() => {
                        log::debug!("EVENT_LOOP: Received message from {:?}", envelope.sender_type);
                        self.handle_message(envelope);
                        // Check if app wants tree rebuild (Elm-style)
                        needs_recompose = self.needs_recompose();
                    }
                }
            }

            Ok(())
        }
    }
}
