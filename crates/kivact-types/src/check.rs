//! Role: Layout validation rules, limits and findings.
use crate::{NodeId, Rect, Role};

/// The box a text must stay inside.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutLimit {
    /// The content box of the text's own node.
    OwnBox,
    /// The padding box of its visual parent: the nearest ancestor with a
    /// background, a border, a clip, or a role that is a single line.
    VisualParent,
}

/// A kind of layout defect.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutRule {
    /// The text reaches outside `into`.
    Overflow { into: LayoutLimit },
    /// A text of a single-line control (a button, a tab, ..) wraps into
    /// `lines` lines.
    Wrapped { lines: u32 },
    /// A single line is meant to sit in the middle of its visual parent
    /// (every box between them is centered) and does not.
    OffCenter,
    /// A clipping ancestor cuts the text, which ends with no ellipsis.
    ClippedWithoutEllipsis,
}

/// What the layout check looks for.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutCheck {
    /// How far (px) a text may reach outside its limit.
    pub overflow_tolerance: f32,
    /// How far (px) the middle of a line may be off the middle of its
    /// visual parent (`None`: the larger of 1 px and 0.08 of the font
    /// size). The layout is in whole pixels, so half a pixel more is
    /// always allowed.
    pub center_tolerance: Option<f32>,
    /// The roles whose text is a single line.
    pub single_line_roles: Vec<Role>,
    /// Nodes with these test ids, and their subtrees, are left out.
    pub ignore_test_ids: Vec<String>,
}

impl Default for LayoutCheck {
    fn default() -> Self {
        LayoutCheck {
            overflow_tolerance: 1.0,
            center_tolerance: None,
            single_line_roles: vec![
                Role::Button,
                Role::Tab,
                Role::MenuItem,
                Role::MenuItemCheckBox,
                Role::MenuItemRadio,
                Role::Option,
                Role::ComboBox,
                Role::Switch,
                Role::Link,
                Role::ColumnHeader,
            ],
            ignore_test_ids: Vec::new(),
        }
    }
}

/// One defect of the text node `node`.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutFinding {
    pub node: NodeId,
    pub rule: LayoutRule,
    /// How far (px) the text is outside its limit, or off center; for
    /// `Wrapped`, the height of the lines after the first.
    pub amount: f32,
    /// The text, in layout space (the window's, before any transform).
    pub text_rect: Rect,
    /// The box it was compared with.
    pub limit_rect: Rect,
}
