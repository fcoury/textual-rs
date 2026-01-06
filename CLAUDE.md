# texrs Development Guide

## Project Overview

texrs is a Rust implementation of Python Textual - a TUI framework for building terminal applications.

## ⚠️ CRITICAL: tmux Session Safety

**NEVER use commands that could kill all tmux sessions.** The user has multiple projects running in tmux sessions.

### SAFEST APPROACH: Do NOT use tmux directly for testing
- Ask the user to manually run and verify TUI output
- Use the `tmux-tui-testing-skill` if available
- Write unit/snapshot tests instead of interactive tmux testing

### If tmux MUST be used (with extreme caution):

1. **NEVER use `tmux kill-server`** - This kills ALL sessions
2. **NEVER use `pkill tmux` or `killall tmux`**
3. **NEVER use `tmux resize-window`** - It can affect the user's current window!
4. **ALWAYS use unique session names** with `$$` (PID) suffix: `SESSION="test-$$"`
5. **ALWAYS verify session exists before operating on it**:
   ```bash
   tmux has-session -t "$SESSION" 2>/dev/null && tmux kill-session -t "$SESSION"
   ```

6. **If a session fails to create, do NOT attempt to kill it**

## Testing TUI Output with tmux

Use the `tmux-tui-testing-skill` to capture ANSI-colored terminal output for comparison testing.

### Basic Pattern

```bash
SESSION="test-$$"
tmux new-session -d -s "$SESSION"
tmux resize-window -t "$SESSION" -x 80 -y 24

# Run the app
tmux send-keys -t "$SESSION" "cargo run --example your_example 2>/dev/null" Enter
sleep 3

# Capture with ANSI escape sequences preserved
tmux capture-pane -t "$SESSION" -p -e

# Cleanup
tmux kill-session -t "$SESSION"
```

### Key Options

- `-d` - Create detached session (headless)
- `-p` - Print captured content to stdout
- `-e` - Preserve ANSI escape sequences (colors, styles)
- Resize with `-x WIDTH -y HEIGHT` for consistent layouts

## Running Python Textual Examples

Python Textual examples are in `~/code/textual`. The shell is fish.

```bash
cd ~/code/textual && source .venv/bin/activate.fish && python3 <path>
```

Example paths:
- `docs/examples/styles/link_background.py`
- `docs/examples/styles/<property_name>.py`

## Comparing Rust vs Python Output

```bash
# Rust version
SESSION="rust-$$"
tmux new-session -d -s "$SESSION"
tmux resize-window -t "$SESSION" -x 60 -y 10
tmux send-keys -t "$SESSION" "cargo run --example link_background 2>/dev/null" Enter
sleep 3
echo "=== RUST ===" && tmux capture-pane -t "$SESSION" -p -e
tmux kill-session -t "$SESSION"

# Python version
SESSION="python-$$"
tmux new-session -d -s "$SESSION"
tmux resize-window -t "$SESSION" -x 60 -y 10
tmux send-keys -t "$SESSION" "cd ~/code/textual && source .venv/bin/activate.fish && python3 docs/examples/styles/link_background.py" Enter
sleep 3
echo "=== PYTHON ===" && tmux capture-pane -t "$SESSION" -p -e
tmux kill-session -t "$SESSION"
```

## ANSI Escape Sequence Reference

Common sequences in captured output:
- `[48;2;R;G;Bm` - 24-bit background color
- `[38;2;R;G;Bm` - 24-bit foreground color
- `[4m` - Underline
- `[1m` - Bold
- `[0m` - Reset all attributes
- `]8;;URL\TEXT]8;;\` - OSC 8 hyperlink

## Running Tests

```bash
# Run all tests
cargo test

# Run specific snapshot test
cargo insta test -p textual --test examples_snapshot -- link_background

# Accept new snapshots
cargo insta test --accept -p textual --test examples_snapshot
```
