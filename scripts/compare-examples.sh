#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/compare-examples.sh [options]

Compare Rust examples (texrs) against Python Textual examples in tmux.
Enumerates Python examples first; only compares when a matching Rust example exists.

Options:
  --width N             Tmux window width (default: 120)
  --height N            Tmux window height (default: 30)
  --sleep SEC           Seconds to wait after launch (default: 1.5)
  --session NAME        Tmux session name (default: compare-examples-<pid>)
  --out DIR             Output directory (default: /tmp/texrs-compare-<session>)
  --ansi                Capture with ANSI escape codes
  --no-clear            Do not clear panes before running
  --no-quit             Do not send q / Ctrl-C after capture
  --no-build            Skip pre-building Rust examples
  --keep                Keep tmux session after finishing
  --show-diff           Print diff to stdout on mismatch
  -h, --help            Show this help

Env overrides:
  TEXRS_DIR             Path to texrs repo (default: pwd)
  TEXTUAL_DIR           Path to textual repo (default: ~/code/textual)
  PY_ACTIVATE           Path to activate script (default: .venv/bin/activate(.fish if present))

Examples:
  scripts/compare-examples.sh
  scripts/compare-examples.sh --width 100 --height 28
USAGE
}

TEXRS_DIR=${TEXRS_DIR:-"$(pwd)"}
TEXTUAL_DIR=${TEXTUAL_DIR:-"$HOME/code/textual"}
WIDTH=120
HEIGHT=30
SLEEP=1.5
SESSION="compare-examples-$$"
OUT_DIR=""
CAPTURE_ANSI=0
DO_CLEAR=1
DO_QUIT=1
KEEP_SESSION=0
SHOW_DIFF=0
SKIP_BUILD=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --width) WIDTH="$2"; shift 2 ;;
    --height) HEIGHT="$2"; shift 2 ;;
    --sleep) SLEEP="$2"; shift 2 ;;
    --session) SESSION="$2"; shift 2 ;;
    --out) OUT_DIR="$2"; shift 2 ;;
    --ansi) CAPTURE_ANSI=1; shift ;;
    --no-clear) DO_CLEAR=0; shift ;;
    --no-quit) DO_QUIT=0; shift ;;
    --no-build) SKIP_BUILD=1; shift ;;
    --keep) KEEP_SESSION=1; shift ;;
    --show-diff) SHOW_DIFF=1; shift ;;
    -h|--help) usage; exit 0 ;;
    --) shift; break ;;
    -*) echo "Unknown option: $1"; usage; exit 2 ;;
    *) echo "Unexpected argument: $1"; usage; exit 2 ;;
  esac
 done

if [[ -z "$OUT_DIR" ]]; then
  OUT_DIR="/tmp/texrs-compare-$SESSION"
fi
mkdir -p "$OUT_DIR"

if [[ ! -d "$TEXRS_DIR" ]]; then
  echo "TEXRS_DIR not found: $TEXRS_DIR" >&2
  exit 1
fi
if [[ ! -d "$TEXTUAL_DIR" ]]; then
  echo "TEXTUAL_DIR not found: $TEXTUAL_DIR" >&2
  exit 1
fi

if [[ $SKIP_BUILD -eq 0 ]]; then
  (cd "$TEXRS_DIR" && cargo build --quiet --examples)
fi

PY_ACTIVATE_DEFAULT=""
shell_name="$(basename "${SHELL:-sh}")"
if [[ "$shell_name" == "fish" && -f "$TEXTUAL_DIR/.venv/bin/activate.fish" ]]; then
  PY_ACTIVATE_DEFAULT="$TEXTUAL_DIR/.venv/bin/activate.fish"
elif [[ -f "$TEXTUAL_DIR/.venv/bin/activate" ]]; then
  PY_ACTIVATE_DEFAULT="$TEXTUAL_DIR/.venv/bin/activate"
elif [[ -f "$TEXTUAL_DIR/.venv/bin/activate.fish" ]]; then
  PY_ACTIVATE_DEFAULT="$TEXTUAL_DIR/.venv/bin/activate.fish"
fi
PY_ACTIVATE=${PY_ACTIVATE:-"$PY_ACTIVATE_DEFAULT"}

mapfile -t py_examples < <(cd "$TEXTUAL_DIR" && ls docs/examples/styles/*.py 2>/dev/null | xargs -n1 basename | sed 's/\.py$//')

if [[ ${#py_examples[@]} -eq 0 ]]; then
  echo "No Python examples found in $TEXTUAL_DIR/docs/examples/styles." >&2
  exit 1
fi

# Setup tmux session and windows.
if tmux has-session -t "$SESSION" 2>/dev/null; then
  echo "Session already exists: $SESSION" >&2
  exit 1
fi

tmux new-session -d -s "$SESSION" -n control

ensure_window() {
  local name="$1"
  if tmux list-windows -t "$SESSION" -F '#{window_name}' | grep -qx "$name"; then
    tmux kill-window -t "$SESSION:$name"
  fi
  tmux new-window -t "$SESSION" -n "$name"
  tmux resize-window -t "$SESSION:$name" -x "$WIDTH" -y "$HEIGHT"
}

cleanup() {
  if [[ $KEEP_SESSION -eq 0 ]]; then
    tmux kill-session -t "$SESSION" >/dev/null 2>&1 || true
  else
    echo "tmux session kept: $SESSION"
    echo "Attach: tmux attach -t $SESSION"
  fi
}
trap cleanup EXIT

capture_cmd() {
  local target="$1"
  local outfile="$2"
  if [[ $CAPTURE_ANSI -eq 1 ]]; then
    tmux capture-pane -t "$SESSION:$target" -p -e > "$outfile"
  else
    tmux capture-pane -t "$SESSION:$target" -p > "$outfile"
  fi
}

run_cmd() {
  local target="$1"
  local cmd="$2"
  if [[ $DO_CLEAR -eq 1 ]]; then
    tmux send-keys -t "$SESSION:$target" "clear" Enter
  fi
  tmux send-keys -t "$SESSION:$target" "$cmd" Enter
}

maybe_quit() {
  local target="$1"
  if [[ $DO_QUIT -eq 1 ]]; then
    tmux send-keys -t "$SESSION:$target" q
    sleep 0.3
    tmux send-keys -t "$SESSION:$target" C-c
  fi
}

status=0
matched=()
failed=()
skipped=()

for ex in "${py_examples[@]}"; do
  ensure_window rust
  ensure_window python

  rust_example=""
  if [[ -f "$TEXRS_DIR/examples/styles/${ex}.rs" ]]; then
    rust_example="styles-${ex}"
  elif [[ -f "$TEXRS_DIR/examples/${ex}.rs" ]]; then
    rust_example="$ex"
  fi
  rust_cmd="cd \"$TEXRS_DIR\" && cargo run --quiet --example \"$rust_example\""
  if [[ -n "$PY_ACTIVATE" ]]; then
    py_cmd="cd \"$TEXTUAL_DIR\" && source \"$PY_ACTIVATE\" && python3 docs/examples/styles/${ex}.py"
  else
    py_cmd="cd \"$TEXTUAL_DIR\" && python3 docs/examples/styles/${ex}.py"
  fi

  if [[ -z "$rust_example" ]]; then
    echo "[skip] rust example not found: $ex"
    skipped+=("$ex")
    continue
  fi

  echo "==> $ex"
  run_cmd rust "$rust_cmd"
  run_cmd python "$py_cmd"
  sleep "$SLEEP"

  rust_out="$OUT_DIR/${ex}.rust.txt"
  py_out="$OUT_DIR/${ex}.python.txt"
  diff_out="$OUT_DIR/${ex}.diff"

  capture_cmd rust "$rust_out"
  capture_cmd python "$py_out"

  if diff -u "$rust_out" "$py_out" > "$diff_out"; then
    echo "OK: $ex"
    matched+=("$ex")
  else
    echo "DIFF: $ex (see $diff_out)"
    status=1
    failed+=("$ex")
    if [[ $SHOW_DIFF -eq 1 ]]; then
      sed -n '1,200p' "$diff_out"
    fi
  fi

  maybe_quit rust
  maybe_quit python
  sleep 0.3

  if [[ $KEEP_SESSION -eq 0 ]]; then
    tmux kill-window -t "$SESSION:rust" >/dev/null 2>&1 || true
    tmux kill-window -t "$SESSION:python" >/dev/null 2>&1 || true
  fi
 done

echo
echo "Matched (${#matched[@]}):"
if [[ ${#matched[@]} -gt 0 ]]; then
  printf '  %s\n' "${matched[@]}"
fi
echo
echo "Failed (${#failed[@]}):"
if [[ ${#failed[@]} -gt 0 ]]; then
  printf '  %s\n' "${failed[@]}"
fi
echo
echo "Skipped (no Rust example) (${#skipped[@]}):"
if [[ ${#skipped[@]} -gt 0 ]]; then
  printf '  %s\n' "${skipped[@]}"
fi

exit $status
