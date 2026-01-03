# AGENTS.md

Project-specific instructions for agents working in this repo.

## Commit Message Guidelines

- Use Angular-style commit messages (e.g., `feat: ...`, `chore: ...`) and include a short detail summary in the body.

## Code hygiene

- After each feature implementation, run `cargo fmt --all` and `cargo clippy`.
  - Fix all the reported issues.

## Example parity checks

- Use `scripts/compare-examples.sh` to compare Rust examples against Python Textual.
- Basic usage: `scripts/compare-examples.sh`
- Useful flags:
  - `--width N --height N` to control tmux window size (defaults 120x30)
  - `--sleep SEC` to wait longer for heavy examples
  - `--show-diff` to print the first part of diffs to stdout
  - `--keep` to keep the tmux session for inspection
- Environment overrides:
  - `TEXRS_DIR` (defaults to current repo)
  - `TEXTUAL_DIR` (defaults to `~/code/textual`)
  - `PY_ACTIVATE` (defaults to detected venv activate script)
- Notes:
  - Script enumerates Python examples and only compares when a matching Rust example exists.
  - `height` example is intentionally different from Python; treat diffs as expected.
  - Outputs captures and diffs under `/tmp/texrs-compare-<session>`.
