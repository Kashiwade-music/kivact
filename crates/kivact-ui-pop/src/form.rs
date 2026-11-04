//! Role: Editable form controls, including platform file drops and pickers.

use crate::Tokens;
use kivact::control::{Node, NodeBuilder, icon, node, text};
use kivact::draw::Shadow;
use kivact::i18n::message;
use kivact::style::current_theme;
use kivact::style::{Align, BorderStyle, ColorToken, Direction, Justify, Theme, style};
use kivact::view::HandlerResult;
use kivact::view::{FilePhase, Handled, IntoText, IntoValue, IntoView, Role, View, callback};
use kivact::{Signal, Value, signal};
use std::path::PathBuf;
use std::rc::Rc;

pub(crate) fn input_style(t: &Theme) -> kivact::style::Style {
    let p = t.get::<Tokens>();
    style()
        .height(36)
        .min_height(0)
        .padding_x(12)
        .padding_y(8)
        .radius(8)
        .border(1, p.line)
        .background(p.surface)
        .color(p.text)
        .font_size(14)
        .line_height(1.2)
        .shadow(Shadow::new(0, 1, 2, kivact::style::rgba(0x0f172a, 0.05)))
        .focus(style().border_color(p.blue).ring(3, ColorToken::Focus))
}
/// The text of a value input while it is being edited. It is the field's
/// content, so a partial entry such as `2027/1` stays as typed; the value
/// follows it only when the text parses. A value that changes elsewhere
/// replaces the text, unless the text already parses to that value.
pub(crate) struct Draft<T: 'static> {
    text: Signal<String>,
    parse: fn(&str) -> Option<T>,
    format: fn(T) -> String,
}

impl<T> Clone for Draft<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Draft<T> {}

impl<T: Copy + PartialEq + 'static> Draft<T> {
    /// Show `value` formatted, and keep following it.
    pub(crate) fn new(
        value: Value<T>,
        parse: fn(&str) -> Option<T>,
        format: fn(T) -> String,
    ) -> Self {
        let draft = Draft {
            text: signal(format(value.get_untracked())),
            parse,
            format,
        };
        kivact::effect(move || {
            let external = value.get();
            if (draft.parse)(&draft.text.get_untracked()) != Some(external) {
                draft.show(external);
            }
        });
        draft
    }

    /// The field's text, which the text input of the date field edits.
    pub(crate) fn text(self) -> Signal<String> {
        self.text
    }

    /// The value `text` stands for, if it is complete.
    pub(crate) fn parse(self, text: &str) -> Option<T> {
        (self.parse)(text)
    }

    /// Replace the text with `value` formatted (a choice made elsewhere).
    pub(crate) fn show(self, value: T) {
        self.text.set((self.format)(value));
    }
}

/// A drop target and platform file-picker trigger. Files are reported to the application.
pub struct Dropzone {
    hint: Value<String>,
    filters: Value<Vec<(String, Vec<String>)>>,
    root: Node,
    on_files: Rc<dyn Fn(Vec<PathBuf>) -> kivact::Result<()>>,
    label: Value<String>,
}
#[track_caller]
pub fn dropzone(label: impl IntoText) -> Dropzone {
    Dropzone {
        hint: String::new().into_value(),
        filters: Vec::new().into_value(),
        root: node().role(Role::Button).focusable(true),
        on_files: Rc::new(|_| Ok(())),
        label: label.into_text_value(),
    }
}
impl Dropzone {
    /// Optional application-provided file guidance, omitted when empty.
    pub fn hint(mut self, hint: impl IntoText) -> Self {
        self.hint = hint.into_text_value();
        self
    }
    /// Application-provided filters for the native file picker.
    pub fn filters(mut self, filters: impl IntoValue<Vec<(String, Vec<String>)>>) -> Self {
        self.filters = filters.into_value();
        self
    }
    /// Receive the dropped or picked files. An `Err` goes to the nearest error handler.
    pub fn on_files<R: HandlerResult>(mut self, f: impl Fn(Vec<PathBuf>) -> R + 'static) -> Self {
        self.on_files = Rc::new(move |files| f(files).into_result());
        self
    }
}
impl NodeBuilder for Dropzone {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Dropzone {
    fn into_view(self) -> View {
        let hovered = signal(false);
        let on_files = self.on_files;
        let callback = callback(move |files| on_files(files));
        let filters = self.filters;
        let hint = self.hint;
        let picker_title = message("pop.dropzone.picker", "Choose files");
        self.root
            .name(self.label)
            .on_file(move |e| {
                hovered.set(matches!(e.phase(), FilePhase::Enter | FilePhase::Over));
                if e.phase() == FilePhase::Drop {
                    callback.call(e.files);
                }
                Handled::Yes
            })
            .on_press(move || {
                let title = picker_title.get();
                let filters = filters.get_untracked();
                // A picker failure goes to the nearest error handler.
                kivact::task::spawn(async move {
                    let picked = kivact::platform::pick_files(kivact::platform::FileDialog {
                        title,
                        filters,
                        ..Default::default()
                    })
                    .await?;
                    if let Some(paths) = picked {
                        callback.call(paths);
                    }
                    Ok::<(), kivact::UiError>(())
                });
            })
            .style(move || {
                let t = &current_theme();
                let p = t.get::<Tokens>();
                style()
                    .direction(Direction::Column)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .padding_x(16)
                    .padding_y(24)
                    .gap(4)
                    .border(1, if hovered.get() { p.blue } else { p.line_strong })
                    .border_style(BorderStyle::Dashed)
                    .radius(10)
                    .background(if hovered.get() {
                        p.hue(crate::Hue::Blue).0
                    } else {
                        p.surface_muted
                    })
                    .hover(style().border_color(p.blue))
            })
            .child(
                node()
                    .style(
                        style()
                            .size(36, 36)
                            .margin_bottom(8)
                            .radius(8)
                            .background(ColorToken::Surface)
                            .border(1, ColorToken::Border)
                            .direction(Direction::Row)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center),
                    )
                    .child(
                        icon("cloud-upload")
                            .style(style().size(24, 24).color(ColorToken::TextMuted)),
                    ),
            )
            .child(text(self.label).style(style().weight(500)))
            .child(kivact::view::show(
                move || !hint.get().is_empty(),
                move || text(hint).style(style().font_size(12).color(ColorToken::TextMuted)),
            ))
            .into_view()
    }
}
