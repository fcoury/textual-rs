//! Core data structures for TCSS stylesheets.
//!
//! This module defines the abstract syntax tree (AST) for parsed TCSS:
//!
//! - [`StyleSheet`]: A collection of CSS rules
//! - [`Rule`]: A selector list paired with declarations
//! - [`Declaration`]: A property-value pair (e.g., `color: red`)
//! - [`Selector`] and related types: The targeting system for rules
//!
//! ## CSS Selector Model
//!
//! TCSS follows the CSS selector model:
//!
//! ```text
//! SelectorList         = ComplexSelector (',' ComplexSelector)*
//! ComplexSelector      = CompoundSelector (Combinator CompoundSelector)*
//! CompoundSelector     = SimpleSelector+
//! SimpleSelector       = Type | Class | Id | Universal | PseudoClass
//! ```
//!
//! For example, `Container > Button.primary, #submit`:
//! - Two complex selectors (comma-separated)
//! - First has two compound selectors with child combinator
//! - Second is a single ID selector

use crate::types::{AlignHorizontal, AlignVertical, BorderEdge, BoxSizing, Display, Dock, Hatch, Keyline, Layout, Overflow, RgbaColor, Scalar, ScrollbarGutter, ScrollbarSize, ScrollbarVisibility, Spacing, TextStyle, Visibility};

/// CSS specificity for determining rule precedence.
///
/// Specificity determines which rule wins when multiple rules match
/// the same element. Higher specificity wins.
///
/// Comparison order: IDs > Classes/PseudoClasses > Types
///
/// # Examples
///
/// - `Button` → (0, 0, 1)
/// - `.primary` → (0, 1, 0)
/// - `#submit` → (1, 0, 0)
/// - `Button.primary#submit` → (1, 1, 1)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    /// Number of ID selectors (`#id`).
    pub ids: u32,
    /// Number of class selectors (`.class`) and pseudo-classes (`:hover`).
    pub classes: u32,
    /// Number of type selectors (`Button`).
    pub types: u32,
}

/// A simple selector that matches elements by a single criterion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    /// Matches elements by type name (e.g., `Button`, `Label`).
    Type(String),
    /// Matches elements by class (e.g., `.primary`, `.active`).
    Class(String),
    /// Matches elements by ID (e.g., `#submit`, `#header`).
    Id(String),
    /// Matches any element (`*`).
    Universal,
    /// Matches elements in a specific state (e.g., `:hover`, `:focus`).
    /// Note: Pseudo-class matching requires runtime state information.
    PseudoClass(String),
    /// The parent selector (`&`) used in nested rules.
    /// Resolves to the parent rule's selector during flattening.
    Parent,
    /// Matches elements by attribute (e.g., `[type=text]`).
    /// The tuple contains (attribute-name, expected-value).
    Attribute(String, String),
}

/// A compound selector: one or more simple selectors without combinators.
///
/// Examples: `Button`, `Button.primary`, `Button.primary#submit`
///
/// All selectors in a compound must match for the compound to match.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundSelector {
    /// The simple selectors that make up this compound.
    pub selectors: Vec<Selector>,
}

impl CompoundSelector {
    /// Creates a new compound selector from a list of simple selectors.
    pub fn new(selectors: Vec<Selector>) -> Self {
        Self { selectors }
    }

    /// Calculates the specificity of this compound selector.
    pub fn specificity(&self) -> Specificity {
        let mut spec = Specificity::default();
        for s in &self.selectors {
            match s {
                Selector::Id(_) => spec.ids += 1,
                Selector::Class(_) | Selector::PseudoClass(_) | Selector::Attribute(_, _) => {
                    spec.classes += 1;
                }
                Selector::Type(_) => spec.types += 1,
                Selector::Universal | Selector::Parent => {}
            }
        }
        spec
    }
}

/// Defines how compound selectors relate to each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combinator {
    /// No combinator (used for the final part of a complex selector).
    None,
    /// Descendant combinator (whitespace): `A B` matches B inside A at any depth.
    Descendant,
    /// Child combinator (`>`): `A > B` matches B that is a direct child of A.
    Child,
    /// Adjacent sibling combinator (`+`): `A + B` matches B immediately after A.
    AdjacentSibling,
    /// General sibling combinator (`~`): `A ~ B` matches B after A (not necessarily adjacent).
    GeneralSibling,
}

/// A part of a complex selector: a compound selector plus its combinator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorPart {
    /// The compound selector for this part.
    pub compound: CompoundSelector,
    /// How this part relates to the next part.
    pub combinator: Combinator,
}

impl SelectorPart {
    /// Creates a new selector part.
    pub fn new(compound: CompoundSelector, combinator: Combinator) -> Self {
        Self {
            compound,
            combinator,
        }
    }
}

/// A complex selector: compound selectors joined by combinators.
///
/// Examples:
/// - `Button` (single compound)
/// - `Container Button` (descendant)
/// - `Container > Button` (child)
/// - `Container > .panel Button.primary` (mixed)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplexSelector {
    /// The parts of this complex selector, from left to right.
    pub parts: Vec<SelectorPart>,
}

impl ComplexSelector {
    /// Creates a new complex selector from parts.
    pub fn new(parts: Vec<SelectorPart>) -> Self {
        Self { parts }
    }

    /// Calculates the total specificity of this complex selector.
    pub fn specificity(&self) -> Specificity {
        self.parts.iter().map(|p| p.compound.specificity()).fold(
            Specificity::default(),
            |acc, x| Specificity {
                ids: acc.ids + x.ids,
                classes: acc.classes + x.classes,
                types: acc.types + x.types,
            },
        )
    }
}

/// A comma-separated list of complex selectors.
///
/// Example: `Button, .link, #submit` contains three complex selectors.
/// A rule matches if any selector in the list matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorList {
    /// The complex selectors in this list.
    pub selectors: Vec<ComplexSelector>,
}

impl SelectorList {
    /// Creates a new selector list.
    pub fn new(selectors: Vec<ComplexSelector>) -> Self {
        Self { selectors }
    }
}

/// A CSS declaration (property-value pair).
///
/// Each variant represents a supported CSS property with its parsed value.
#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    /// The `color` property for text color.
    Color(RgbaColor),
    /// The `background` property for background color.
    Background(RgbaColor),
    /// The `tint` property for color overlay (applies to both fg and bg).
    /// Example: `tint: magenta 40%;` blends 40% magenta over all colors.
    Tint(RgbaColor),
    /// The `background-tint` property for background color overlay.
    /// Example: `background-tint: white 50%;` blends 50% white over background.
    BackgroundTint(RgbaColor),
    /// The `width` property for element width.
    Width(Scalar),
    /// The `height` property for element height.
    Height(Scalar),
    /// The `max-height` property for maximum element height.
    MaxHeight(Scalar),
    /// The `max-width` property for maximum element width.
    MaxWidth(Scalar),
    /// The `min-height` property for minimum element height.
    MinHeight(Scalar),
    /// The `min-width` property for minimum element width.
    MinWidth(Scalar),
    /// The `margin` property for outer spacing.
    Margin(Spacing),
    /// The `margin-top` property for top outer spacing.
    MarginTop(Scalar),
    /// The `margin-right` property for right outer spacing.
    MarginRight(Scalar),
    /// The `margin-bottom` property for bottom outer spacing.
    MarginBottom(Scalar),
    /// The `margin-left` property for left outer spacing.
    MarginLeft(Scalar),
    /// The `padding` property for inner spacing.
    Padding(Spacing),
    /// The `border` property for element borders.
    Border(BorderEdge),

    // Scrollbar properties
    /// The `scrollbar-color` property for scrollbar thumb color.
    ScrollbarColor(RgbaColor),
    /// The `scrollbar-color-hover` property for scrollbar thumb hover color.
    ScrollbarColorHover(RgbaColor),
    /// The `scrollbar-color-active` property for scrollbar thumb active color.
    ScrollbarColorActive(RgbaColor),
    /// The `scrollbar-background` property for scrollbar track color.
    ScrollbarBackground(RgbaColor),
    /// The `scrollbar-background-hover` property for scrollbar track hover color.
    ScrollbarBackgroundHover(RgbaColor),
    /// The `scrollbar-background-active` property for scrollbar track active color.
    ScrollbarBackgroundActive(RgbaColor),
    /// The `scrollbar-corner-color` property for scrollbar corner color.
    ScrollbarCornerColor(RgbaColor),
    /// The `scrollbar-size` property (horizontal, vertical thickness).
    ScrollbarSize(ScrollbarSize),
    /// The `scrollbar-size-horizontal` property.
    ScrollbarSizeHorizontal(u16),
    /// The `scrollbar-size-vertical` property.
    ScrollbarSizeVertical(u16),
    /// The `scrollbar-gutter` property (auto or stable).
    ScrollbarGutter(ScrollbarGutter),
    /// The `scrollbar-visibility` property (visible or hidden).
    ScrollbarVisibility(ScrollbarVisibility),

    // Box model properties
    /// The `box-sizing` property (content-box or border-box).
    BoxSizing(BoxSizing),

    // Display properties
    /// The `display` property (block or none).
    Display(Display),
    /// The `visibility` property (visible or hidden).
    Visibility(Visibility),
    /// The `opacity` property (0.0 to 1.0, or 0% to 100%).
    /// Controls how the widget's background blends with the parent background.
    Opacity(f64),

    // Overflow properties
    /// The `overflow-x` property for horizontal overflow behavior.
    OverflowX(Overflow),
    /// The `overflow-y` property for vertical overflow behavior.
    OverflowY(Overflow),

    // Layout and Grid properties
    /// The `layout` property (vertical, horizontal, grid).
    Layout(Layout),
    /// The `dock` property (top, bottom, left, right).
    Dock(Dock),
    /// The `layers` property defining available layer names for a container.
    /// Layers control rendering order: lower indices render first (bottom), higher on top.
    /// Example: `layers: below above;` defines "below" as index 0, "above" as index 1.
    Layers(Vec<String>),
    /// The `layer` property assigning a widget to a named layer.
    /// Example: `layer: above;` assigns the widget to the "above" layer.
    Layer(String),
    /// The `grid-size` property (columns, optional rows).
    GridSize(u16, Option<u16>),
    /// The `grid-columns` property (column width definitions).
    GridColumns(Vec<Scalar>),
    /// The `grid-rows` property (row height definitions).
    GridRows(Vec<Scalar>),
    /// The `grid-gutter` property (vertical, optional horizontal spacing).
    GridGutter(Scalar, Option<Scalar>),
    /// The `column-span` property (child spans multiple columns).
    ColumnSpan(u16),
    /// The `row-span` property (child spans multiple rows).
    RowSpan(u16),

    // Link properties
    /// The `link-color` property for link text color.
    LinkColor(RgbaColor),
    /// The `link-color-hover` property for link text color on hover.
    LinkColorHover(RgbaColor),
    /// The `link-background` property for link background color.
    LinkBackground(RgbaColor),
    /// The `link-background-hover` property for link background color on hover.
    LinkBackgroundHover(RgbaColor),
    /// The `link-style` property for link text style (bold, underline, etc.).
    LinkStyle(TextStyle),
    /// The `link-style-hover` property for link text style on hover.
    LinkStyleHover(TextStyle),

    // Content alignment properties (text alignment within widget)
    /// The `content-align-horizontal` property for horizontal content alignment.
    ContentAlignHorizontal(AlignHorizontal),
    /// The `content-align-vertical` property for vertical content alignment.
    ContentAlignVertical(AlignVertical),
    /// The `content-align` shorthand property for both horizontal and vertical alignment.
    ContentAlign(AlignHorizontal, AlignVertical),

    // Container alignment properties (child positioning within container)
    /// The `align-horizontal` property for horizontal child positioning.
    AlignHorizontal(AlignHorizontal),
    /// The `align-vertical` property for vertical child positioning.
    AlignVertical(AlignVertical),
    /// The `align` shorthand property for both horizontal and vertical child positioning.
    Align(AlignHorizontal, AlignVertical),

    // Border title/subtitle properties
    /// The `border-title-align` property for title alignment in top border.
    BorderTitleAlign(AlignHorizontal),
    /// The `border-subtitle-align` property for subtitle alignment in bottom border.
    BorderSubtitleAlign(AlignHorizontal),
    /// The `border-title-color` property for title text color.
    BorderTitleColor(RgbaColor),
    /// The `border-subtitle-color` property for subtitle text color.
    BorderSubtitleColor(RgbaColor),
    /// The `border-title-background` property for title background color.
    BorderTitleBackground(RgbaColor),
    /// The `border-subtitle-background` property for subtitle background color.
    BorderSubtitleBackground(RgbaColor),
    /// The `border-title-style` property for title text style (bold, italic, etc.).
    BorderTitleStyle(TextStyle),
    /// The `border-subtitle-style` property for subtitle text style (bold, italic, etc.).
    BorderSubtitleStyle(TextStyle),

    // Edge-specific border properties
    /// The `border-top` property for top edge only.
    BorderTop(BorderEdge),
    /// The `border-bottom` property for bottom edge only.
    BorderBottom(BorderEdge),
    /// The `border-left` property for left edge only.
    BorderLeft(BorderEdge),
    /// The `border-right` property for right edge only.
    BorderRight(BorderEdge),

    // Outline properties (non-layout-affecting border overlay)
    /// The `outline` property for all edges (overlay, doesn't affect layout).
    Outline(BorderEdge),
    /// The `outline-top` property for top edge outline.
    OutlineTop(BorderEdge),
    /// The `outline-right` property for right edge outline.
    OutlineRight(BorderEdge),
    /// The `outline-bottom` property for bottom edge outline.
    OutlineBottom(BorderEdge),
    /// The `outline-left` property for left edge outline.
    OutlineLeft(BorderEdge),

    // Hatch pattern fill
    /// The `hatch` property for filling an area with a pattern.
    /// Syntax: `hatch: <pattern> <color> [opacity]`
    Hatch(Hatch),

    // Keyline (box-drawing borders around widgets)
    /// The `keyline` property for drawing box borders around widgets in a container.
    /// Syntax: `keyline: <style> <color>` where style is none, thin, heavy, or double.
    Keyline(Keyline),

    // Offset properties (visual position adjustment after layout)
    /// The `offset` property for X and Y visual position offset.
    /// Syntax: `offset: <x> <y>` where values are scalars (cells, %, etc.).
    /// Positive X moves right, positive Y moves down.
    Offset(Scalar, Scalar),
    /// The `offset-x` property for horizontal position offset.
    OffsetX(Scalar),
    /// The `offset-y` property for vertical position offset.
    OffsetY(Scalar),

    /// An unrecognized property (stored for forward compatibility).
    Unknown(String),
}

/// An item inside a rule block: either a declaration or a nested rule.
#[derive(Clone, Debug, PartialEq)]
pub enum RuleItem {
    /// A property-value declaration.
    Declaration(Declaration),
    /// A nested rule (for `&` parent selector support).
    NestedRule(Rule),
}

/// A CSS rule: a selector list paired with declarations.
///
/// Example:
/// ```css
/// Button, .link {
///     color: red;
///     width: 100%;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// The selectors that determine which elements this rule applies to.
    pub selectors: SelectorList,
    /// The declarations and nested rules inside this rule.
    pub items: Vec<RuleItem>,
}

impl Rule {
    /// Creates a new rule with the given selectors and items.
    pub fn new(selectors: SelectorList, items: Vec<RuleItem>) -> Self {
        Self { selectors, items }
    }

    /// Returns only the declarations from this rule (excludes nested rules).
    ///
    /// Useful for the cascade and testing.
    pub fn declarations(&self) -> Vec<Declaration> {
        self.items
            .iter()
            .filter_map(|item| match item {
                RuleItem::Declaration(decl) => Some(decl.clone()),
                RuleItem::NestedRule(_) => None,
            })
            .collect()
    }
}

/// A complete TCSS stylesheet containing multiple rules.
#[derive(Clone, Debug, Default)]
pub struct StyleSheet {
    /// The rules in this stylesheet, in source order.
    pub rules: Vec<Rule>,
}
