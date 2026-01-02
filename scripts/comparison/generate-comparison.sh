#!/bin/bash
# generate-comparison.sh - Generate Python vs Rust comparison images for textual-rs
# Usage: ./generate-comparison.sh [example_count]

set -e

# Get script directory and ensure we run from there
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# =============================================================================
# Configuration
# =============================================================================
EXAMPLE_COUNT="${1:-10}"  # Default to 10 examples
PYTHON_DIR="$HOME/code/textual/docs/examples/styles"
RUST_DIR="$HOME/code/texrs"
VHS_OUTPUT_DIR="/tmp/vhs_captures"
OUTPUT_DIR="$SCRIPT_DIR/output"
TAPE_DIR="$SCRIPT_DIR"  # Keep tapes in main dir so relative paths work

# VHS settings
TERMINAL_WIDTH=800
TERMINAL_HEIGHT=500
FONT_SIZE=14

# Curated examples (visually interesting)
CURATED_EXAMPLES=(
  "align_all"
  "border_all"
  "content_align_all"
  "grid_columns"
  "color"
  "margin_all"
  "opacity"
  "hatch"
  "outline_all"
  "dock_all"
  "background"
  "height_comparison"
)

# =============================================================================
# Setup
# =============================================================================
mkdir -p "$OUTPUT_DIR"
mkdir -p "$VHS_OUTPUT_DIR"

echo "=== textual-rs Comparison Image Generator ==="
echo "Examples to capture: $EXAMPLE_COUNT"
echo "Output directory: $OUTPUT_DIR"
echo ""

# =============================================================================
# Step 1: Get list of ported examples
# =============================================================================
echo "Finding ported examples..."

# Get intersection of Python and Rust examples
PYTHON_EXAMPLES=$(ls "$PYTHON_DIR"/*.py 2>/dev/null | xargs -n1 basename | sed 's/.py$//' | sort)
RUST_EXAMPLES=$(ls "$RUST_DIR/examples"/*.rs 2>/dev/null | xargs -n1 basename | sed 's/.rs$//' | sort)

# Find common examples
PORTED_EXAMPLES=()
for ex in $PYTHON_EXAMPLES; do
  if echo "$RUST_EXAMPLES" | grep -q "^${ex}$"; then
    PORTED_EXAMPLES+=("$ex")
  fi
done

echo "Found ${#PORTED_EXAMPLES[@]} ported examples"

# =============================================================================
# Step 2: Select examples to use
# =============================================================================
SELECTED_EXAMPLES=()
count=0

# First, add curated examples that are ported
for ex in "${CURATED_EXAMPLES[@]}"; do
  if [[ " ${PORTED_EXAMPLES[*]} " =~ " ${ex} " ]]; then
    SELECTED_EXAMPLES+=("$ex")
    ((count++)) || true
    if [ $count -ge $EXAMPLE_COUNT ]; then
      break
    fi
  fi
done

# If we need more, add from ported list
if [ $count -lt $EXAMPLE_COUNT ]; then
  for ex in "${PORTED_EXAMPLES[@]}"; do
    if [[ ! " ${SELECTED_EXAMPLES[*]} " =~ " ${ex} " ]]; then
      SELECTED_EXAMPLES+=("$ex")
      ((count++)) || true
      if [ $count -ge $EXAMPLE_COUNT ]; then
        break
      fi
    fi
  done
fi

echo "Selected ${#SELECTED_EXAMPLES[@]} examples:"
printf '  - %s\n' "${SELECTED_EXAMPLES[@]}"
echo ""

# =============================================================================
# Step 3: Pre-build Rust examples
# =============================================================================
echo "Pre-building Rust examples..."
if [ -f "$RUST_DIR/target/release/examples/${SELECTED_EXAMPLES[0]}" ]; then
  echo "Examples already built."
else
  (cd "$RUST_DIR" && cargo build --examples --release) || {
    echo "Warning: Build failed, but continuing with existing binaries"
  }
  echo "Build complete."
fi
echo ""

# =============================================================================
# Step 4: Generate VHS tape files
# =============================================================================
echo "Generating VHS tape files..."

generate_python_tape() {
  local name=$1
  local tape_file="$TAPE_DIR/${name}_python.tape"

  cat > "$tape_file" << EOF
# VHS tape for Python example: $name
Output "$VHS_OUTPUT_DIR/${name}_python.gif"
Set Shell "bash"
Set FontSize $FONT_SIZE
Set Width $TERMINAL_WIDTH
Set Height $TERMINAL_HEIGHT
Set Theme "Dracula"
Set TypingSpeed 0
Set Padding 10

Type "cd '$PYTHON_DIR' && source ~/code/textual/.venv/bin/activate && python ${name}.py"
Enter
Sleep 3s
Sleep 1s
Type "q"
Sleep 500ms
EOF
  echo "$tape_file"
}

generate_rust_tape() {
  local name=$1
  local tape_file="$TAPE_DIR/${name}_rust.tape"

  cat > "$tape_file" << EOF
# VHS tape for Rust example: $name
Output "$VHS_OUTPUT_DIR/${name}_rust.gif"
Set Shell "bash"
Set FontSize $FONT_SIZE
Set Width $TERMINAL_WIDTH
Set Height $TERMINAL_HEIGHT
Set Theme "Dracula"
Set TypingSpeed 0
Set Padding 10

Type "cd '$RUST_DIR' && ./target/release/examples/${name}"
Enter
Sleep 4s
Sleep 1s
Type "q"
Sleep 500ms
EOF
  echo "$tape_file"
}

# Generate tapes for all selected examples
for ex in "${SELECTED_EXAMPLES[@]}"; do
  generate_python_tape "$ex" > /dev/null
  generate_rust_tape "$ex" > /dev/null
  echo "  Generated tapes for: $ex"
done

echo ""

# =============================================================================
# Step 5: Run VHS to capture screenshots
# =============================================================================
echo "Capturing screenshots with VHS..."
echo "(This may take a while...)"
echo ""

for ex in "${SELECTED_EXAMPLES[@]}"; do
  echo "Capturing $ex..."

  # Capture Python version
  if [ ! -f "$OUTPUT_DIR/${ex}_python.png" ]; then
    echo "  Python..."
    # Run VHS from /tmp for reliability
    (cd /tmp && vhs "$TAPE_DIR/${ex}_python.tape") || {
      echo "  Warning: Failed to capture Python $ex"
    }
    sleep 3  # Delay between captures
  else
    echo "  Python (cached)"
  fi

  # Capture Rust version
  if [ ! -f "$OUTPUT_DIR/${ex}_rust.png" ]; then
    echo "  Rust..."
    # Run VHS from /tmp for reliability
    (cd /tmp && vhs "$TAPE_DIR/${ex}_rust.tape") || {
      echo "  Warning: Failed to capture Rust $ex"
    }
    sleep 3  # Delay between captures
  else
    echo "  Rust (cached)"
  fi
done

echo ""

# Extract frames from GIFs and copy to output directory
echo "Extracting frames from GIF captures..."
for gif in "$VHS_OUTPUT_DIR"/*.gif; do
  if [ -f "$gif" ]; then
    base=$(basename "$gif" .gif)
    # Coalesce GIF frames and extract a late frame (80% through)
    frame_count=$(magick identify "$gif" 2>/dev/null | wc -l)
    target_frame=$((frame_count * 80 / 100))
    if [ "$target_frame" -lt 1 ]; then target_frame=0; fi

    magick "$gif" -coalesce -delete 0-$((target_frame - 1)) -delete 1--1 "$OUTPUT_DIR/${base}.png" 2>/dev/null && \
      echo "  Extracted: $base" || \
      echo "  Failed: $base"
  fi
done
echo ""

# =============================================================================
# Step 6: Stitch images together
# =============================================================================
echo "Stitching images..."

# Create labeled pairs
for ex in "${SELECTED_EXAMPLES[@]}"; do
  python_img="$OUTPUT_DIR/${ex}_python.png"
  rust_img="$OUTPUT_DIR/${ex}_rust.png"

  if [ -f "$python_img" ] && [ -f "$rust_img" ]; then
    # Add "Python" label
    magick "$python_img" \
      -gravity North -background '#282a36' -splice 0x35 \
      -font Helvetica -pointsize 18 -fill '#f8f8f2' \
      -annotate +0+8 'Python' \
      "$OUTPUT_DIR/${ex}_python_labeled.png"

    # Add "Rust" label
    magick "$rust_img" \
      -gravity North -background '#282a36' -splice 0x35 \
      -font Helvetica -pointsize 18 -fill '#f8f8f2' \
      -annotate +0+8 'Rust' \
      "$OUTPUT_DIR/${ex}_rust_labeled.png"

    # Combine side-by-side with example name
    magick "$OUTPUT_DIR/${ex}_python_labeled.png" "$OUTPUT_DIR/${ex}_rust_labeled.png" \
      +append \
      -gravity South -background '#282a36' -splice 0x25 \
      -font Helvetica -pointsize 14 -fill '#6272a4' \
      -annotate +0+5 "$ex" \
      "$OUTPUT_DIR/${ex}_pair.png"

    echo "  Created pair: $ex"
  else
    echo "  Skipping $ex (missing images)"
  fi
done

echo ""

# =============================================================================
# Step 7: Create final montage
# =============================================================================
echo "Creating final montage..."

# Collect all pair images
PAIR_IMAGES=()
for ex in "${SELECTED_EXAMPLES[@]}"; do
  if [ -f "$OUTPUT_DIR/${ex}_pair.png" ]; then
    PAIR_IMAGES+=("$OUTPUT_DIR/${ex}_pair.png")
  fi
done

if [ ${#PAIR_IMAGES[@]} -gt 0 ]; then
  # Stack pairs in a grid (2 columns)
  magick montage "${PAIR_IMAGES[@]}" \
    -tile 2x \
    -geometry +10+10 \
    -background '#282a36' \
    "$OUTPUT_DIR/comparison_grid.png"

  # Add header
  magick "$OUTPUT_DIR/comparison_grid.png" \
    -gravity North -background '#282a36' -splice 0x70 \
    -font Helvetica-Bold -pointsize 32 -fill '#f8f8f2' \
    -annotate +0+15 'textual-rs: Python Textual to Rust' \
    -font Helvetica -pointsize 18 -fill '#6272a4' \
    -annotate +0+50 '48 examples ported • github.com/fcoury/textual-rs' \
    "$OUTPUT_DIR/textual-rs-comparison.png"

  # Optimize for Twitter (max 4096x4096, <5MB)
  magick "$OUTPUT_DIR/textual-rs-comparison.png" \
    -resize '4096x4096>' \
    -quality 92 \
    "$OUTPUT_DIR/textual-rs-comparison-twitter.png"

  echo ""
  echo "=== Done! ==="
  echo "Final image: $OUTPUT_DIR/textual-rs-comparison-twitter.png"

  # Show file size
  ls -lh "$OUTPUT_DIR/textual-rs-comparison-twitter.png"
else
  echo "Error: No pair images created!"
  exit 1
fi
