//! Role: Place retained views in a shaped paragraph.

use std::rc::Rc;

use kivact_reactive::memo;
use kivact_style::draw::{InlineParagraph, Point, TextStyle};
use kivact_style::{Length, Position, pct, style};
use kivact_wire::{InlineBox, Size};

use crate::control::{Node, NodeBuilder, node};
use crate::view::{View, node_ref};

/// Mount atomic views at UTF-8 byte offsets in a wrapped paragraph.
/// Each view is measured at its intrinsic width, capped at the paragraph
/// width. Its bottom edge aligns with the text baseline. Call during setup.
pub fn inline_view(
    text: impl Into<String>,
    typography: TextStyle,
    children: impl IntoIterator<Item = (usize, View)>,
) -> Node {
    let text = text.into();
    let paragraph = node_ref();
    let bounds = paragraph.rect();
    let width = memo(move || bounds.get().map(|bounds| bounds.width));
    let children: Vec<_> = children.into_iter().collect();
    let references: Vec<_> = children.iter().map(|_| node_ref()).collect();
    let measurements: Vec<_> = children
        .iter()
        .zip(&references)
        .map(|((offset, _), reference)| {
            let rect = reference.rect();
            let size = memo(move || {
                rect.get().map(|rect| Size {
                    width: rect.width,
                    height: rect.height,
                })
            });
            (*offset, size)
        })
        .collect();
    let source = text.clone();
    let layout = memo(move || {
        let width = width.get()?;
        let boxes = measurements
            .iter()
            .enumerate()
            .map(|(index, (offset, bounds))| {
                let size = bounds.get()?;
                Some(InlineBox {
                    id: index as u64,
                    offset: *offset,
                    size,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        super::shape_inline_paragraph(&source, &typography, Some(width), &boxes)
            .map(|paragraph| Snapshot(Rc::new(paragraph)))
    });
    let children: Vec<_> = children
        .into_iter()
        .zip(references)
        .enumerate()
        .map(|(index, ((_, child), reference))| {
            node().node_ref(reference).child(child).style(move || {
                let point = layout.get().and_then(|snapshot| {
                    snapshot
                        .0
                        .boxes()
                        .iter()
                        .find(|item| item.id == index as u64)
                        .map(|item| (item.rect.x, item.rect.y))
                });
                let (x, y) = point.unwrap_or_default();
                style()
                    .position(Position::Absolute)
                    .width(Length::MaxContent)
                    .max_width(pct(100))
                    .left(x)
                    .top(y)
            })
        })
        .collect();
    node()
        .name(text)
        .node_ref(paragraph)
        .child(children)
        .style(move || {
            let height = layout
                .get()
                .map_or(0.0, |snapshot| snapshot.0.paragraph().size().height);
            style().width(pct(100)).height(height).shrink(0)
        })
        .draw(move |draw, _| {
            if let Some(snapshot) = layout.get() {
                draw.paragraph(
                    snapshot.0.paragraph(),
                    Point::default(),
                    kivact_style::ColorToken::Text,
                );
            }
        })
}

#[derive(Clone)]
struct Snapshot(Rc<InlineParagraph>);

impl PartialEq for Snapshot {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
