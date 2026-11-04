//! Role: Editable fields, steppers, tags and native file drop targets.
use kivact::i18n::message;
use kivact::style::current_theme;
use std::path::PathBuf;
use std::rc::Rc;

use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::draw::{Layer, Path, Point};
use kivact::style::{Align, BorderStyle, ColorToken, Direction, Position, pct, style};
use kivact::task::spawn;
use kivact::view::{FilePhase, Handled, HandlerResult, IntoText, IntoValue, Role};
use kivact::{Signal, Value, memo, signal};

use crate::Tokens;

/// Icons or captions placed inside a field without replacing native editing.
#[track_caller]
pub fn input_adorned(
    prefix: impl kivact::view::IntoView,
    control: kivact::ui::TextInput,
    suffix: impl kivact::view::IntoView,
) -> Node {
    row((
        control.style(style().width(pct(100)).padding_left(34).padding_right(52)),
        node().child(prefix).style(
            style()
                .position(Position::Absolute)
                .left(10)
                .align_self(Align::Center)
                .color(ColorToken::TextMuted),
        ),
        node().child(suffix).style(
            style()
                .position(Position::Absolute)
                .right(8)
                .align_self(Align::Center)
                .color(ColorToken::TextMuted),
        ),
    ))
    .style(style().width(pct(100)).align_items(Align::Center))
}

/// Fixed prefix and suffix around an editable field.
#[track_caller]
pub fn input_group(
    prefix: impl IntoText,
    control: kivact::ui::TextInput,
    suffix: impl IntoText,
) -> Node {
    input_group_content(crate::mono(prefix), control, crate::mono(suffix))
}

/// Fixed prefix and suffix slots accepting icons or arbitrary public views.
#[track_caller]
pub fn input_group_content(
    prefix: impl kivact::view::IntoView,
    control: kivact::ui::TextInput,
    suffix: impl kivact::view::IntoView,
) -> Node {
    let addon = |content: kivact::view::View| {
        node()
            .style(move || {
                let t = &current_theme();
                style()
                    .height(t.get::<Tokens>().control_height)
                    .padding_x(10)
                    .align_items(Align::Center)
                    .background(t.get::<Tokens>().surface_3)
                    .border(1, t.get::<Tokens>().line_strong)
            })
            .child(content)
    };
    row((
        addon(prefix.into_view()),
        control.style(style().grow(1).min_width(0)),
        addon(suffix.into_view()),
    ))
    .style(style().width(pct(100)).align_items(Align::Center))
}

/// Native file picker and OS drop target. Upload transport remains application-owned.
#[track_caller]
pub fn dropzone(label: impl IntoText) -> Dropzone {
    let label = label.into_text_value();
    let over = signal(false);
    Dropzone {
        root: column(())
            .role(Role::Button)
            .name(label)
            .focusable(true)
            .style(move || {
                let t = &current_theme();
                style()
                    .direction(Direction::Column)
                    .align_items(Align::Center)
                    .gap(6)
                    .padding_y(22)
                    .padding_x(16)
                    .border(1.5, t.get::<Tokens>().line_strong)
                    .border_style(BorderStyle::Dashed)
                    .selected(
                        style()
                            .background(t.get::<Tokens>().primary_weak)
                            .border_color(t.colors().accent),
                    )
            }),
        over,
        hint: String::new().into_value(),
        filters: Vec::new().into_value(),
        on_files: Rc::new(|_| Ok(())),
        label,
    }
}
/// What the drop zone is doing, and what it last failed at.
#[derive(Clone, PartialEq)]
enum PickStatus {
    Ready,
    /// The native picker is open.
    Picking,
    /// The last drop or pick was refused; shown under the label.
    Failed(String),
}

/// Native drop target with application-owned guidance and file filters.
pub struct Dropzone {
    root: Node,
    label: Value<String>,
    over: Signal<bool>,
    hint: Value<String>,
    filters: Value<Vec<(String, Vec<String>)>>,
    on_files: Rc<dyn Fn(Vec<PathBuf>) -> kivact::Result<()>>,
}
impl Dropzone {
    /// Receive dropped and picked files; no action by default. An `Err` is
    /// shown under the label until the next drop or pick.
    pub fn on_files<R: HandlerResult>(mut self, f: impl Fn(Vec<PathBuf>) -> R + 'static) -> Self {
        self.on_files = Rc::new(move |files| f(files).into_outcome().map(|_| ()));
        self
    }

    /// Optional file guidance; empty guidance has no node.
    pub fn hint(mut self, hint: impl IntoText) -> Self {
        self.hint = hint.into_text_value();
        self
    }
    /// Filters passed unchanged to the native picker.
    pub fn filters(mut self, filters: impl IntoValue<Vec<(String, Vec<String>)>>) -> Self {
        self.filters = filters.into_value();
        self
    }
}
impl NodeBuilder for Dropzone {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl kivact::view::IntoView for Dropzone {
    fn into_view(self) -> kivact::view::View {
        let on_files = self.on_files;
        let over = self.over;
        let hint = self.hint;
        let filters = self.filters;
        let status = signal(PickStatus::Ready);
        let drop_callback = on_files.clone();
        let click_callback = on_files;
        let picker_title = message("game.dropzone.picker", "Choose files");
        self.root
            .child((
                kivact::control::icon("cloud-upload")
                    .style(style().size(28, 28).color(ColorToken::Accent)),
                text(self.label).style(style().weight(700)),
                kivact::view::show(
                    move || !hint.get().is_empty(),
                    move || text(hint).style(style().font_size(11.5).color(ColorToken::TextMuted)),
                ),
            ))
            .on_file(move |e| {
                match e.phase {
                    FilePhase::Enter | FilePhase::Over => over.set(true),
                    FilePhase::Leave => over.set(false),
                    FilePhase::Drop => {
                        over.set(false);
                        status.set(match drop_callback(e.files) {
                            Ok(()) => PickStatus::Ready,
                            Err(failure) => PickStatus::Failed(failure.to_string()),
                        });
                    }
                }
                Ok(Handled::Yes)
            })
            .on_press(move || {
                if status.get_untracked() == PickStatus::Picking {
                    return;
                }
                status.set(PickStatus::Picking);
                let callback = click_callback.clone();
                let title = picker_title.get();
                let filters = filters.get_untracked();
                spawn(async move {
                    match kivact::platform::pick_files(kivact::platform::FileDialog {
                        title,
                        filters,
                        ..Default::default()
                    })
                    .await
                    {
                        Ok(Some(paths)) => status.set(match callback(paths) {
                            Ok(()) => PickStatus::Ready,
                            Err(failure) => PickStatus::Failed(failure.to_string()),
                        }),
                        Ok(None) => status.set(PickStatus::Ready),
                        Err(failure) => status.set(PickStatus::Failed(failure.to_string())),
                    }
                });
            })
            .selected(over)
            .draw(|d, s| {
                d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                    let mut x = -s.height;
                    while x < s.width {
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(x, s.height))
                                .line_to(Point::new(x + 10., s.height))
                                .line_to(Point::new(x + s.height + 10., 0.))
                                .line_to(Point::new(x + s.height, 0.))
                                .close(),
                            kivact::style::rgba(0x1283f0, 0.04),
                        );
                        x += 20.;
                    }
                });
            })
            .child(kivact::view::show_some(
                memo(move || match status.get() {
                    PickStatus::Failed(message) => Some(message),
                    _ => None,
                }),
                |message| {
                    text(message)
                        .role(Role::Alert)
                        .style(style().color(ColorToken::Danger))
                },
            ))
            .into_view()
    }
}
