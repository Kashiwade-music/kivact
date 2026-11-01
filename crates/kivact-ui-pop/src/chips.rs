//! Role: Removable chips with application-owned additions.
use kivact::ui::{ButtonVariant, ControlSize, button};

use kivact::control::{Node, NodeBuilder, icon, row, text};
use kivact::i18n::{message, message_with};
use kivact::style::{Align, BorderStyle, ColorToken, style};
use kivact::view::{HandlerResult, IntoText, IntoValue, IntoView, View, callback, for_index};
use kivact::{Callback, Value};

/// A chip that calls its owner when its remove button is invoked.
#[track_caller]
pub fn removable_chip<F, R>(label: impl IntoText, remove: F) -> Node
where
    F: Fn() -> R + 'static,
    R: HandlerResult,
{
    let label = label.into_text_value();
    row((
        text(label),
        button(message_with(
            "pop.chip.remove",
            "Remove {label}",
            move || [("label", label.get())],
        ))
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(icon("x"))
        .style(style().size(16, 16))
        .on_press(remove),
    ))
    .style(
        style()
            .gap(4)
            .height(26)
            .padding_x(10)
            .align_items(Align::Center)
            .background(ColorToken::SurfaceAlt)
            .radius(6)
            .font_size(13),
    )
}

/// A chip collection that reports requests to its owner.
pub struct ChipList {
    root: Node,
    tags: Value<Vec<String>>,
    on_add: Callback<()>,
    on_remove: Callback<usize>,
}
#[track_caller]
pub fn chip_list(tags: impl IntoValue<Vec<String>>) -> ChipList {
    ChipList {
        root: row(()).style(style().gap(6).wrap(true)),
        tags: tags.into_value(),
        on_add: callback(|_| {}),
        on_remove: callback(|_| {}),
    }
}
impl ChipList {
    /// Request an application-defined addition.
    pub fn on_add<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_add = callback(move |_| f());
        self
    }
    /// Request removal of an item by index.
    pub fn on_remove<R: HandlerResult>(mut self, f: impl Fn(usize) -> R + 'static) -> Self {
        self.on_remove = callback(f);
        self
    }
}
impl NodeBuilder for ChipList {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for ChipList {
    fn into_view(self) -> View {
        let remove = self.on_remove;
        let add = self.on_add;
        self.root
            .child((
                for_index(self.tags, move |index, label| {
                    removable_chip(label, move || remove.call(index))
                }),
                button(message("pop.chip.add", "Add"))
                    .icon(kivact::control::icon("plus"))
                    .size(ControlSize::Small)
                    .variant(ButtonVariant::Ghost)
                    .on_press(move || add.call(()))
                    .style(
                        style()
                            .height(26)
                            .border(1, ColorToken::Border)
                            .border_style(BorderStyle::Dashed),
                    ),
            ))
            .into_view()
    }
}
