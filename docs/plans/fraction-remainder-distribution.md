# Plan: Add Fraction Type + Fix Grid Remainder Distribution

## Problem
1. Current grid code distributes leftover pixels to **first** tracks; Textual uses **later** tracks
2. Float arithmetic accumulates rounding errors over many iterations
3. Fraction-based math is used throughout Textual's layout system (10+ files)

## Solution: Custom Fraction Type (with num-rational alternative documented)

**Primary**: Custom minimal Fraction - zero dependencies, tailored to our needs
**Alternative**: Include commented-out `num-rational` version for easy future switch

## Implementation Plan

### Step 1: Create `crates/textual/src/fraction.rs`

```rust
//! Rational number type for precise layout calculations.
//!
//! Avoids floating-point accumulation errors in layout distribution.

use std::ops::{Add, Mul};

/// A rational number (fraction) for precise arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fraction {
    num: i64,
    den: i64,
}

impl Fraction {
    pub const ZERO: Fraction = Fraction { num: 0, den: 1 };

    pub fn new(num: i64, den: i64) -> Self {
        if den == 0 {
            return Self::ZERO;
        }
        let g = gcd(num.abs(), den.abs());
        let sign = if den < 0 { -1 } else { 1 };
        Self {
            num: sign * num / g,
            den: sign * den / g,
        }
    }

    /// Integer part (floor division).
    pub fn floor(&self) -> i64 {
        if self.num >= 0 {
            self.num / self.den
        } else {
            (self.num - self.den + 1) / self.den
        }
    }

    /// Fractional part (always non-negative).
    pub fn fract(&self) -> Fraction {
        let floor = self.floor();
        Fraction::new(self.num - floor * self.den, self.den)
    }
}

impl From<i32> for Fraction {
    fn from(n: i32) -> Self {
        Fraction::new(n as i64, 1)
    }
}

impl Add for Fraction {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Fraction::new(
            self.num * rhs.den + rhs.num * self.den,
            self.den * rhs.den,
        )
    }
}

impl Mul for Fraction {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Fraction::new(self.num * rhs.num, self.den * rhs.den)
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_positive() {
        assert_eq!(Fraction::new(7, 3).floor(), 2);
        assert_eq!(Fraction::new(6, 3).floor(), 2);
    }

    #[test]
    fn floor_negative() {
        assert_eq!(Fraction::new(-7, 3).floor(), -3);
    }

    #[test]
    fn fract_returns_remainder() {
        let f = Fraction::new(7, 3); // 2 + 1/3
        assert_eq!(f.fract(), Fraction::new(1, 3));
    }

    #[test]
    fn remainder_distribution() {
        // Simulate distributing 25 among 2 equal parts
        let portion = Fraction::new(25, 2);
        let mut remainder = Fraction::ZERO;
        let mut sizes = vec![0i64; 2];

        for size in &mut sizes {
            let raw = portion + remainder;
            *size = raw.floor();
            remainder = raw.fract();
        }

        // Extra pixel goes to LAST element (Textual behavior)
        assert_eq!(sizes, vec![12, 13]);
        assert_eq!(sizes.iter().sum::<i64>(), 25);
    }
}

// =============================================================================
// ALTERNATIVE: num-rational crate
// =============================================================================
// To switch to num-rational, add to Cargo.toml:
//   num-rational = "0.4"
//
// Then replace this entire file with:
// ```
// pub use num_rational::Rational64 as Fraction;
//
// pub trait FractionExt {
//     const ZERO: Self;
// }
//
// impl FractionExt for Fraction {
//     const ZERO: Self = Fraction::new_raw(0, 1);
// }
// ```
// =============================================================================
```

### Step 2: Update `crates/textual/src/lib.rs`

Add module declaration:
```rust
pub mod fraction;
pub use fraction::Fraction;
```

### Step 3: Update `crates/textual/src/containers/grid.rs`

Replace the fr distribution loop with Fraction-based math:

```rust
use crate::Fraction;

// In distribute_space(), replace lines 144-166:
if total_fr > 0.0 {
    let portion = Fraction::new(remaining as i64, (total_fr * 1000.0) as i64);
    let mut remainder = Fraction::ZERO;

    for (i, spec) in track_specs.iter().enumerate() {
        if spec.unit == Unit::Fraction {
            let fr_scaled = Fraction::new((spec.value * 1000.0) as i64, 1);
            let raw = portion * fr_scaled + remainder;
            sizes[i] = raw.floor() as i32;
            remainder = raw.fract();
        }
    }
} else if auto_count > 0 {
    let portion = Fraction::new(remaining as i64, auto_count as i64);
    let mut remainder = Fraction::ZERO;

    for (i, spec) in track_specs.iter().enumerate() {
        if spec.unit == Unit::Auto {
            let raw = portion + remainder;
            sizes[i] = raw.floor() as i32;
            remainder = raw.fract();
        }
    }
} else if specs.is_empty() {
    let portion = Fraction::new(available_for_tracks as i64, count as i64);
    let mut remainder = Fraction::ZERO;

    for size in &mut sizes {
        let raw = portion + remainder;
        *size = raw.floor() as i32;
        remainder = raw.fract();
    }
}

// REMOVE the leftover distribution code (lines 175-182) - no longer needed
```

## Files to Modify
1. **Create**: `crates/textual/src/fraction.rs` - New Fraction type
2. **Edit**: `crates/textual/src/lib.rs` - Add module export
3. **Edit**: `crates/textual/src/containers/grid.rs` - Use Fraction in `distribute_space`

## Future Use
The Fraction type will be reusable in:
- CSS scalar resolution (%, fr, vw, vh)
- Box model calculations
- Other container layouts (Horizontal, Vertical)
- Min/max width/height calculations
