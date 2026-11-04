//! Role: Select or drop native paths and validate them before publishing.
use crate::recipes_field::frame;
use crate::theme_access::tokens;
use kivact::custom::PartState;
use kivact::i18n::message;
use kivact::platform::FileDialog;
use kivact::platform::pick_files;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::memo;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Justify;
use kivact::style::style;
use kivact::task::spawn;
use kivact::ui::ButtonVariant;
use kivact::ui::button;
use kivact::view::FileEvent;
use kivact::view::FilePhase;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::show;
use kivact::view::text;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct FileInput {
    root: Node,
    label: Value<String>,
    value: Binding<Vec<PathBuf>>,
    extensions: Rc<Vec<String>>,
    max_bytes: Option<u64>,
    multiple: Value<bool>,
    dropzone: Value<bool>,
}

pub fn file_input(label: impl IntoText, value: impl IntoBinding<Vec<PathBuf>>) -> FileInput {
    FileInput {
        root: node(),
        label: label.into_text_value(),
        value: value.into_binding(),
        extensions: Rc::default(),
        max_bytes: None,
        multiple: (false).into_value(),
        dropzone: (false).into_value(),
    }
}

impl FileInput {
    /// File extensions accepted by both the picker and dropzone.
    pub fn extensions(mut self, extensions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extensions = Rc::new(
            extensions
                .into_iter()
                .map(|extension| {
                    extension
                        .into()
                        .trim_start_matches('.')
                        .to_ascii_lowercase()
                })
                .collect(),
        );
        self
    }

    pub fn max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }

    pub fn multiple(mut self, multiple: impl IntoValue<bool>) -> Self {
        self.multiple = multiple.into_value();
        self
    }

    pub fn dropzone(mut self, dropzone: impl IntoValue<bool>) -> Self {
        self.dropzone = dropzone.into_value();
        self
    }
}

impl NodeBuilder for FileInput {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

fn extension_matches(path: &Path, extensions: &[String]) -> bool {
    extensions.is_empty()
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extensions
                    .iter()
                    .any(|allowed| allowed.eq_ignore_ascii_case(extension))
            })
}

impl IntoView for FileInput {
    fn into_view(self) -> View {
        let Self {
            mut root,
            label,
            value,
            extensions,
            max_bytes,
            multiple,
            dropzone,
        } = self;
        let disabled = root.accessible_disabled();
        let disabled = memo(move || disabled.as_ref().is_some_and(|disabled| disabled()));
        let pending = signal(false);
        let dragging = signal(false);
        let status = signal(None::<String>);
        let invalid_file = message(
            "soft.file_input.invalid_file",
            "One or more files are not allowed.",
        );
        let too_large = message(
            "soft.file_input.too_large",
            "One or more files exceed the size limit or could not be read.",
        );
        let choose_one = message("soft.file_input.choose_one", "Choose one file.");
        let accept = Rc::new({
            let extensions = extensions.clone();
            move |files: Vec<PathBuf>| {
                let error = if !multiple.get() && files.len() > 1 {
                    Some(choose_one.get())
                } else if files
                    .iter()
                    .any(|file| !extension_matches(file, &extensions))
                {
                    Some(invalid_file.get())
                } else if max_bytes.is_some_and(|max| {
                    files.iter().any(|file| {
                        std::fs::metadata(file)
                            .map_or(true, |metadata| !metadata.is_file() || metadata.len() > max)
                    })
                }) {
                    Some(too_large.get())
                } else {
                    None
                };
                if error.is_none() && !files.is_empty() {
                    value.set(files);
                }
                status.set(error);
            }
        });
        let name = memo(move || {
            value.with(|files| {
                files
                    .iter()
                    .map(|path| {
                        path.file_name()
                            .unwrap_or(path.as_os_str())
                            .to_string_lossy()
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            })
        });
        let anchor = node_ref();
        let focus = anchor.focus_within();
        let trigger_ref = node_ref();
        let alert_ref = node_ref();
        let trigger = button(label)
            .variant(ButtonVariant::Ghost)
            .hide_label(true)
            .icon(icon("upload"))
            .disabled(move || disabled || pending.get())
            .invalid(move || status.get().is_some())
            .node_ref(trigger_ref)
            .described_by(alert_ref)
            .child(text(move || {
                if name.get().is_empty() {
                    label.get()
                } else {
                    name.get()
                }
            }))
            .style(style().grow(1).justify_content(Justify::Start))
            .on_press({
                let accept = accept.clone();
                move || {
                    if disabled.get_untracked() || pending.get_untracked() {
                        return;
                    }
                    pending.set(true);
                    let accept = accept.clone();
                    let request = FileDialog {
                        title: label.get_untracked(),
                        filters: if extensions.is_empty() {
                            Vec::new()
                        } else {
                            vec![(label.get_untracked(), extensions.as_ref().clone())]
                        },
                        ..FileDialog::default()
                    };
                    spawn(async move {
                        let result = if multiple.get() {
                            pick_files(request).await
                        } else {
                            kivact::platform::pick_file(request)
                                .await
                                .map(|file| file.map(|file| vec![file]))
                        };
                        pending.set(false);
                        if !disabled.get_untracked() {
                            match result {
                                Ok(Some(files)) => accept(files),
                                Ok(None) => {}
                                Err(error) => status.set(Some(error.to_string())),
                            }
                            trigger_ref.restore_focus();
                        }
                    });
                }
            });
        let clear = show(
            move || !name.get().is_empty(),
            move || {
                button(message("soft.file_input.clear", "Clear files"))
                    .variant(ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(icon("x"))
                    .disabled(move || disabled || pending.get())
                    .on_press(move || {
                        value.set(Vec::new());
                        status.set(None);
                        trigger_ref.restore_focus();
                    })
            },
        );
        let control = row((trigger, clear))
            .node_ref(anchor)
            .style(move || {
                let t = tokens();
                let s = frame(
                    &t,
                    &PartState {
                        focused: focus.get(),
                        ..PartState::default()
                    },
                    status.get().is_some(),
                )
                .align_items(Align::Center)
                .min_height(if dropzone.get() { 96 } else { 32 });
                if dragging.get() || focus.get() {
                    s.border_color(t.colors.accent)
                        .shadow(t.shadows.input_focus_ring)
                } else {
                    s
                }
            })
            .on_file(move |event: FileEvent| {
                if !dropzone.get() || disabled.get_untracked() || pending.get_untracked() {
                    return Handled::No;
                }
                dragging.set(matches!(event.phase, FilePhase::Enter | FilePhase::Over));
                if event.phase == FilePhase::Drop {
                    accept(event.files);
                }
                Handled::Yes
            });
        let alert = show(
            move || status.get().is_some(),
            move || {
                text(move || status.get().unwrap_or_default())
                    .node_ref(alert_ref)
                    .role(Role::Alert)
                    .style(move || style().font_size(12).color(tokens().colors.text_error))
            },
        );
        root.child(column((control, alert)).style(style().gap(4)))
            .into_view()
    }
}
