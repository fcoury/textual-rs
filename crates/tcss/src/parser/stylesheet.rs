use crate::types::{BorderEdge, RgbaColor, Scalar, Spacing};

/// CSS specificity for determining rule precedence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub ids: u32,
    pub classes: u32,
    pub types: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    Type(String),
    Class(String),
    Id(String),
    Universal,
    PseudoClass(String),
    Parent,
    Attribute(String, String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundSelector {
    pub selectors: Vec<Selector>,
}

impl CompoundSelector {
    pub fn new(selectors: Vec<Selector>) -> Self {
        Self { selectors }
    }

    pub fn specificity(&self) -> Specificity {
        let mut spec = Specificity::default();
        for s in &self.selectors {
            match s {
                Selector::Id(_) => spec.ids += 1,
                // Attributes have the same specificity as classes and pseudo-classes
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Combinator {
    None,
    Descendant,
    Child,
    AdjacentSibling, // +
    GeneralSibling,  // ~
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorPart {
    pub compound: CompoundSelector,
    pub combinator: Combinator,
}

impl SelectorPart {
    pub fn new(compound: CompoundSelector, combinator: Combinator) -> Self {
        Self {
            compound,
            combinator,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComplexSelector {
    pub parts: Vec<SelectorPart>,
}

impl ComplexSelector {
    pub fn new(parts: Vec<SelectorPart>) -> Self {
        Self { parts }
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorList {
    pub selectors: Vec<ComplexSelector>,
}

impl SelectorList {
    pub fn new(selectors: Vec<ComplexSelector>) -> Self {
        Self { selectors }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Declaration {
    Color(RgbaColor),
    Background(RgbaColor),
    Width(Scalar),
    Height(Scalar),
    Margin(Spacing),
    Padding(Spacing),
    Border(BorderEdge),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub selectors: SelectorList,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn new(selectors: SelectorList, declarations: Vec<Declaration>) -> Self {
        Self {
            selectors,
            declarations,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyleSheet {
    pub rules: Vec<Rule>,
}
