//! Rational number type for precise layout calculations.
//!
//! Avoids floating-point accumulation errors in layout distribution.
//! Used throughout the layout system for pixel-perfect remainder handling.

use std::ops::{Add, Mul};

/// A rational number (fraction) for precise arithmetic.
///
/// Layout calculations involve divisions that don't produce integers.
/// Floating-point accumulates errors over many iterations, but Fraction
/// maintains exact precision by keeping numerator and denominator separate.
///
/// # Example
///
/// ```
/// use textual::Fraction;
///
/// // Distributing 25 pixels among 2 equal parts
/// let portion = Fraction::new(25, 2);  // 12.5
/// let mut remainder = Fraction::ZERO;
/// let mut sizes = Vec::new();
///
/// for _ in 0..2 {
///     let raw = portion + remainder;
///     sizes.push(raw.floor());
///     remainder = raw.fract();
/// }
///
/// // Extra pixel goes to LAST element (Textual behavior)
/// assert_eq!(sizes, vec![12, 13]);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fraction {
    num: i64,
    den: i64,
}

impl Fraction {
    /// Zero as a fraction (0/1).
    pub const ZERO: Fraction = Fraction { num: 0, den: 1 };

    /// Create a new fraction, automatically reduced to lowest terms.
    ///
    /// Handles negative denominators by moving sign to numerator.
    /// Returns ZERO if denominator is zero.
    pub fn new(num: i64, den: i64) -> Self {
        if den == 0 {
            return Self::ZERO;
        }
        let g = gcd(num.abs(), den.abs());
        let sign = if den < 0 { -1 } else { 1 };
        Self {
            num: sign * num / g,
            den: (sign * den / g).abs(),
        }
    }

    /// Integer part (floor division).
    ///
    /// For positive fractions, this is simply num / den.
    /// For negative fractions, rounds toward negative infinity.
    pub fn floor(&self) -> i64 {
        if self.num >= 0 {
            self.num / self.den
        } else {
            // For negative: -7/3 = -3 (not -2)
            (self.num - self.den + 1) / self.den
        }
    }

    /// Fractional part (always non-negative, less than 1).
    ///
    /// Returns the remainder after extracting the floor.
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

impl From<i64> for Fraction {
    fn from(n: i64) -> Self {
        Fraction::new(n, 1)
    }
}

impl Add for Fraction {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Fraction::new(self.num * rhs.den + rhs.num * self.den, self.den * rhs.den)
    }
}

impl Mul for Fraction {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Fraction::new(self.num * rhs.num, self.den * rhs.den)
    }
}

/// Greatest common divisor using Euclidean algorithm.
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
    fn new_reduces_to_lowest_terms() {
        assert_eq!(Fraction::new(6, 9), Fraction::new(2, 3));
        assert_eq!(Fraction::new(100, 50), Fraction::new(2, 1));
    }

    #[test]
    fn new_handles_negative_denominator() {
        let f = Fraction::new(3, -4);
        assert_eq!(f.num, -3);
        assert_eq!(f.den, 4);
    }

    #[test]
    fn new_handles_zero_denominator() {
        assert_eq!(Fraction::new(5, 0), Fraction::ZERO);
    }

    #[test]
    fn floor_positive() {
        assert_eq!(Fraction::new(7, 3).floor(), 2); // 2.33... -> 2
        assert_eq!(Fraction::new(6, 3).floor(), 2); // 2.0 -> 2
        assert_eq!(Fraction::new(1, 3).floor(), 0); // 0.33... -> 0
    }

    #[test]
    fn floor_negative() {
        assert_eq!(Fraction::new(-7, 3).floor(), -3); // -2.33... -> -3
        assert_eq!(Fraction::new(-6, 3).floor(), -2); // -2.0 -> -2
    }

    #[test]
    fn fract_returns_remainder() {
        let f = Fraction::new(7, 3); // 2 + 1/3
        assert_eq!(f.fract(), Fraction::new(1, 3));
    }

    #[test]
    fn fract_of_integer_is_zero() {
        let f = Fraction::new(6, 3); // exactly 2
        assert_eq!(f.fract(), Fraction::ZERO);
    }

    #[test]
    fn add_fractions() {
        let a = Fraction::new(1, 3);
        let b = Fraction::new(1, 6);
        assert_eq!(a + b, Fraction::new(1, 2));
    }

    #[test]
    fn mul_fractions() {
        let a = Fraction::new(2, 3);
        let b = Fraction::new(3, 4);
        assert_eq!(a * b, Fraction::new(1, 2));
    }

    #[test]
    fn remainder_distribution_two_parts() {
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

    #[test]
    fn remainder_distribution_three_parts() {
        // Simulate distributing 100 among 3 equal parts
        let portion = Fraction::new(100, 3);
        let mut remainder = Fraction::ZERO;
        let mut sizes = vec![0i64; 3];

        for size in &mut sizes {
            let raw = portion + remainder;
            *size = raw.floor();
            remainder = raw.fract();
        }

        // 100 / 3 = 33.33... -> [33, 33, 34]
        assert_eq!(sizes, vec![33, 33, 34]);
        assert_eq!(sizes.iter().sum::<i64>(), 100);
    }

    #[test]
    fn remainder_distribution_with_fractions() {
        // Simulate grid-columns: 1fr 2fr 1fr with 170 pixels
        // total_fr = 4, so portion = 170/4 = 42.5
        let total = 170i64;
        let fractions = [1i64, 2, 1];
        let total_fr: i64 = fractions.iter().sum();

        let mut remainder = Fraction::ZERO;
        let mut sizes = vec![0i64; 3];

        for (i, &fr) in fractions.iter().enumerate() {
            let raw = Fraction::new(total * fr, total_fr) + remainder;
            sizes[i] = raw.floor();
            remainder = raw.fract();
        }

        // 1fr=42, 2fr=85, 1fr=43 (extra goes to last)
        assert_eq!(sizes, vec![42, 85, 43]);
        assert_eq!(sizes.iter().sum::<i64>(), 170);
    }
}

// =============================================================================
// ALTERNATIVE: num-rational crate
// =============================================================================
// To switch to num-rational, add to Cargo.toml:
//   num-rational = "0.4"
//
// Then replace this entire file with:
//
// ```rust
// //! Re-export num-rational as our Fraction type.
//
// pub use num_rational::Rational64 as Fraction;
//
// /// Extension trait for Fraction constants.
// pub trait FractionExt {
//     /// Zero constant.
//     const ZERO: Self;
// }
//
// impl FractionExt for Fraction {
//     const ZERO: Self = Fraction::new_raw(0, 1);
// }
// ```
// =============================================================================
