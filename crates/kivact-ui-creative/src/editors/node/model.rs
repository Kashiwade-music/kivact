//! Role: Data models of the node editor: sockets, nodes, controls and links.

use kivact::draw::Pixels;
use kivact::geometry::{Point, Size};
use kivact::style::{Color, rgb};

/// A labeled typed socket. Its index in the node's list is its identity.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeSocket {
    pub label: String,
    pub type_name: String,
    pub color: Color,
    /// Optional vertical placement within the node, in logical pixels.
    pub offset: Option<f64>,
}
impl NodeSocket {
    pub fn new(label: impl Into<String>, type_name: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            type_name: type_name.into(),
            color,
            offset: None,
        }
    }
    pub fn at(mut self, offset: f64) -> Self {
        if offset.is_finite() && offset >= 30.0 {
            self.offset = Some(offset);
        }
        self
    }
}

/// Consumer-owned node presentation. Parameters are display strings supplied by the application.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeData {
    pub id: u64,
    pub label: String,
    pub position: Point,
    pub size: Size,
    pub color: Color,
    pub inputs: Vec<NodeSocket>,
    pub outputs: Vec<NodeSocket>,
    pub parameters: Vec<String>,
    /// Editable controls are stored in the application's node model.
    pub controls: Vec<NodeControl>,
    // xtask-allow(state-fields): Independent optional application-owned image and mute flag.
    /// Application-provided image displayed within the node preview rectangle.
    pub preview: Option<Pixels>,
    // xtask-allow(state-fields): Independent optional application-owned image and mute flag.
    pub muted: bool,
}
impl Default for NodeData {
    fn default() -> Self {
        Self {
            id: 0,
            label: String::new(),
            position: Point::new(0.0, 0.0),
            size: Size::new(180.0, 160.0),
            color: rgb(0x8b7cff),
            inputs: Vec::new(),
            outputs: Vec::new(),
            parameters: Vec::new(),
            controls: Vec::new(),
            preview: None,
            muted: false,
        }
    }
}
impl NodeData {
    pub fn new(
        id: u64,
        label: impl Into<String>,
        inputs: Vec<NodeSocket>,
        outputs: Vec<NodeSocket>,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            position: Point::new(0.0, 0.0),
            size: Size::new(180.0, 160.0),
            color: rgb(0x8b7cff),
            inputs,
            outputs,
            ..Self::default()
        }
    }
    pub fn position(mut self, position: Point) -> Self {
        self.position = position;
        self
    }
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
    pub fn parameters(mut self, parameters: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.parameters = parameters.into_iter().map(Into::into).collect();
        self
    }
    /// Display application-provided pixels, contained within the preview rectangle.
    pub fn preview(mut self, preview: Pixels) -> Self {
        self.preview = Some(preview);
        self
    }
    pub fn controls(mut self, controls: Vec<NodeControl>) -> Self {
        self.controls = controls;
        self
    }
}

/// A node parameter with explicit placement and an accessible editable value.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeControl {
    pub label: String,
    pub offset: f64,
    pub value: NodeControlValue,
}
#[derive(Clone, Debug, PartialEq)]
pub enum NodeControlValue {
    Number {
        value: f64,
        min: f64,
        max: f64,
        step: f64,
    },
    Choice {
        options: Vec<String>,
        selected: usize,
    },
    Toggle(bool),
}
impl NodeControl {
    pub fn number(
        label: impl Into<String>,
        offset: f64,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        Self {
            label: label.into(),
            offset,
            value: NodeControlValue::Number {
                value,
                min,
                max,
                step,
            },
        }
    }
    pub fn choice(
        label: impl Into<String>,
        offset: f64,
        options: impl IntoIterator<Item = impl Into<String>>,
        selected: usize,
    ) -> Self {
        Self {
            label: label.into(),
            offset,
            value: NodeControlValue::Choice {
                options: options.into_iter().map(Into::into).collect(),
                selected,
            },
        }
    }
    /// A toggle control. A constant: `checked` is plain data inside the node's
    /// control list, so a changed state is a new list.
    pub fn toggle(label: impl Into<String>, offset: f64, checked: bool) -> Self {
        Self {
            label: label.into(),
            offset,
            value: NodeControlValue::Toggle(checked),
        }
    }
    pub(crate) fn change(&mut self, requested: Option<f64>, delta: f64) {
        match &mut self.value {
            NodeControlValue::Number {
                value,
                min,
                max,
                step,
            } => {
                let next = requested.unwrap_or(*value + delta * *step);
                if next.is_finite() && min.is_finite() && max.is_finite() && *min <= *max {
                    *value = next.clamp(*min, *max)
                }
            }
            NodeControlValue::Choice { options, selected } => {
                if !options.is_empty() {
                    *selected = (*selected + 1) % options.len()
                }
            }
            NodeControlValue::Toggle(value) => *value = !*value,
        }
    }
}

/// A directed connection between two explicit socket indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeLink {
    pub from: u64,
    pub output: usize,
    pub to: u64,
    pub input: usize,
}
impl NodeLink {
    pub const fn new(from: u64, output: usize, to: u64, input: usize) -> Self {
        Self {
            from,
            output,
            to,
            input,
        }
    }
}

/// A proposed change to application-owned node data.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeRequest {
    /// Absolute positions computed from the beginning of the gesture.
    Move(Vec<(u64, Point)>),
    /// Request a connection without deduplication or input replacement.
    Connect(NodeLink),
    /// Request removal of this connection.
    Disconnect(NodeLink),
    /// Request removal of selected nodes and their incident links.
    Delete {
        nodes: Vec<u64>,
        links: Vec<NodeLink>,
    },
    /// Request toggling mute on these nodes.
    ToggleMute(Vec<u64>),
    /// An absolute proposed control value.
    Control {
        node: u64,
        index: usize,
        value: NodeControlValue,
    },
}
