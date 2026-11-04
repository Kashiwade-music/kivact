//! Role: Share touch routing and sheet chrome among Soft selectors.
use crate::{DialogPurpose, SheetHeight, bottom_sheet, heading};
use kivact::control::{NodeBuilder, column};
use kivact::draw::Point;
use kivact::style::{pct, style};
use kivact::view::{
    IntoView, MouseButton, NodeRef, PointerEvent, PointerId, PointerKind, PointerPhase,
};
use kivact::{Binding, Value, var};

/// How touch activation presents a Selector's list. Keyboard and mouse keep
/// the anchored popup on every setting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectorPresentation {
    #[default]
    Popover,
    BottomSheet,
    Adaptive,
}

impl SelectorPresentation {
    pub(super) fn uses_sheet(self) -> bool {
        match self {
            Self::Popover => false,
            Self::BottomSheet => true,
            Self::Adaptive => {
                let viewport = kivact::platform::window_size();
                viewport.width < 768.0 || viewport.height < 500.0
            }
        }
    }
}

pub(super) fn sheet<V: IntoView>(
    label: Value<String>,
    open: impl Into<Binding<bool>>,
    trigger: NodeRef,
    body: impl Fn() -> V + 'static,
) -> impl IntoView {
    bottom_sheet(label, open)
        .purpose(DialogPurpose::Info)
        .height(SheetHeight::Hug)
        .padding(0)
        .final_focus(trigger)
        .content(move || {
            column((
                heading(3, label).style(
                    style()
                        .font_size(19)
                        .line_height(28.0 / 19.0)
                        .weight(700)
                        .margin_x(12)
                        .margin_bottom(8),
                ),
                body(),
            ))
            .style(style().width(pct(100)).padding_top(16).padding_x(4))
        })
}

/// A primary touch release on its original trigger, without scrolling.
pub(super) fn touch_tap(anchor: NodeRef) -> impl Fn(PointerEvent, bool) -> bool + Copy {
    let contact = var(None::<(PointerId, Point, bool)>);
    let rect = anchor.rect();
    move |event, eligible| {
        if let PointerPhase::Down(press) = event.phase() {
            contact.set(
                (eligible
                    && event.kind() == PointerKind::Touch
                    && event.is_primary()
                    && press.button == MouseButton::Primary)
                    .then_some((event.id(), event.window_pos(), false)),
            );
            return false;
        }
        let Some((id, start, moved)) = contact.get().filter(|(id, _, _)| *id == event.id()) else {
            return false;
        };
        let pos = event.window_pos();
        let moved = moved || (pos.x - start.x).hypot(pos.y - start.y) > 10.0;
        match event.phase() {
            PointerPhase::Move => contact.set(Some((id, start, moved))),
            PointerPhase::Cancel => contact.set(None),
            PointerPhase::Up(_) => {
                contact.set(None);
                return eligible && !moved && rect.get().is_some_and(|rect| rect.contains(pos));
            }
            _ => {}
        }
        false
    }
}
