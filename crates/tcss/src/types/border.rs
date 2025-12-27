use crate::types::color::RgbaColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderKind {
    #[default]
    None,
    Ascii,
    Blank,
    Block,
    Double,
    Dashed,
    Heavy,
    Hidden,
    Outer,
    Inner,
    Solid,
    Round,
    Thick,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BorderEdge {
    pub kind: BorderKind,
    pub color: Option<RgbaColor>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Border {
    pub top: BorderEdge,
    pub right: BorderEdge,
    pub bottom: BorderEdge,
    pub left: BorderEdge,
}

impl Border {
    pub fn all(edge: BorderEdge) -> Self {
        Self {
            top: edge.clone(),
            right: edge.clone(),
            bottom: edge.clone(),
            left: edge,
        }
    }

    pub fn is_none(&self) -> bool {
        self.top.kind == BorderKind::None
            && self.right.kind == BorderKind::None
            && self.bottom.kind == BorderKind::None
            && self.left.kind == BorderKind::None
    }
}
