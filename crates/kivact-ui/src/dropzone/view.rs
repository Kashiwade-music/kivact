//! Role: The dropzone component, file picking, validation and status rendering.

use crate::dropzone::look::DropzoneTheme;
use kivact_core::custom::Caller;
use kivact_core::platform::{FileDialog, pick_file, pick_files};
use kivact_core::state::{Callback, Value};
use kivact_core::task::spawn;
use kivact_core::view::{Element, FilePhase, Handled, IntoView, Node, Role, View, node, text};
use kivact_interaction::{DropzoneState, FileInfo, use_dropzone};
use kivact_macros::component;
use std::rc::Rc;

/// A validated file drop target labelled by `label`. Native file metadata
/// stays application-owned.
///
/// Options: `icon(view)` adds decorative content above the label, mounted
/// once; `hint(text)` shows application-provided guidance; `on_files(f)`
/// receives valid files (a callback error appears in the drop target);
/// `validate(f)` rejects a file with a user-facing error message;
/// `multiple(true)` accepts several files in one operation (without it, more
/// than one file shows the message `dropzone.one_file`, "Choose one file.");
/// `pick_on_press(true)` opens a native picker on pointer, keyboard or
/// assistive activation; `accept_drop(..)` accepts drag and drop input
/// independently of the native picker (a change to false ends the active
/// drag-over visit without clearing files); `filters(..)` passes filters to
/// the native picker.
#[component]
pub fn dropzone(
    label: Value<String>,
    icon: Option<View>,
    #[prop(default = String::new())] hint: Value<String>,
    #[prop(default = Vec::new())] filters: Value<Vec<(String, Vec<String>)>>,
    #[prop(default = false)] multiple: bool,
    #[prop(default = true)] accept_drop: Value<bool>,
    #[prop(default = false)] pick_on_press: bool,
    validate: Option<Rc<dyn Fn(&FileInfo) -> Option<String>>>,
    on_files: Option<Callback<Vec<FileInfo>>>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let disabled = caller.disabled();
    let disabled = kivact_core::state::memo(move || disabled.is_some_and(|d| d.get()));
    let validate = validate.unwrap_or_else(|| Rc::new(|_: &FileInfo| None));
    let mut api = use_dropzone()
        .validate(move |file| validate(file))
        .on_files(move |files| match on_files {
            // A failed callback becomes the visible failed state.
            Some(on_files) => on_files.try_call(files),
            None => Ok(()),
        });
    if multiple {
        api = api.multiple();
    }
    // A policy change is a real leave event into the shared drop machine,
    // so a rejected drag cannot retain stale hover feedback.
    kivact_core::state::effect_on(accept_drop, move |enabled| {
        if !enabled {
            api.leave();
        }
    });
    let state = api.state();
    let reference = kivact_core::view::node_ref();
    let status_ref = kivact_core::view::node_ref();
    let root = if pick_on_press {
        node().part(kivact_interaction::use_button().control())
    } else {
        node().with_node(|n| n.role(Role::Group))
    };
    let (label_view, hint_view) = captions(label, hint, state, &variants);
    crate::layer::with_base_recipe_style(
        root,
        |r: &DropzoneTheme| r.root_recipe(),
        &variants,
        move || state.get(),
    )
    .node_ref(reference)
    .name(label)
    .invalid(move || matches!(state.get(), DropzoneState::Failed(_)))
    .on_file(move |event| file_event(api, accept_drop, disabled, event))
    .on_press(move || {
        pick(
            api,
            disabled,
            pick_on_press,
            label,
            filters,
            multiple,
            reference,
        )
    })
    .child((
        icon.map(|icon| {
            crate::layer::with_recipe_style(
                node().child(icon),
                |r: &DropzoneTheme| r.icon_recipe(),
                &variants,
                move || state.get(),
            )
        }),
        label_view,
        hint_view,
        failure_status(state, &variants),
        drag_status(state, &variants, status_ref),
    ))
}

/// Run a native file-picking transaction and restore focus on completion.
fn pick(
    api: kivact_interaction::DropzoneApi,
    disabled: Value<bool>,
    pick_on_press: bool,
    label: Value<String>,
    filters: Value<Vec<(String, Vec<String>)>>,
    multiple: bool,
    reference: kivact_core::view::NodeRef,
) {
    if !pick_on_press || disabled.get_untracked() || !api.begin_pick() {
        return;
    }
    let request = FileDialog {
        title: label.get_untracked(),
        filters: filters.get_untracked(),
        ..FileDialog::default()
    };
    spawn(async move {
        let result = if multiple {
            pick_files(request).await
        } else {
            pick_file(request)
                .await
                .map(|file| file.map(|file| vec![file]))
        };
        match result {
            Ok(Some(files)) => {
                if disabled.get_untracked() {
                    api.cancel_pick()
                } else {
                    api.accept(files)
                }
            }
            Ok(None) => api.cancel_pick(),
            Err(error) => api.fail(error),
        }
        reference.restore_focus();
    });
}

/// Expose validation and picker failures through the alert recipe outlet.
fn failure_status(state: Value<DropzoneState>, variants: &kivact_core::view::Variants) -> View {
    kivact_core::view::show_some(
        move || match state.get() {
            DropzoneState::Failed(error) => Some(error.to_string()),
            _ => None,
        },
        {
            let variants = variants.clone();
            move |message: Value<String>| {
                crate::layer::with_recipe_style(
                    text(message),
                    |r: &DropzoneTheme| r.status_recipe(),
                    &variants,
                    move || state.get(),
                )
                .with_node(|n| n.role(Role::Alert))
            }
        },
    )
    .into_view()
}

/// Expose current picking and drag-validation feedback through the status outlet.
fn drag_status(
    state: Value<DropzoneState>,
    variants: &kivact_core::view::Variants,
    status_ref: kivact_core::view::NodeRef,
) -> Node {
    crate::layer::with_recipe_style(
        text(move || match state.get() {
            DropzoneState::Failed(_) => String::new(),
            DropzoneState::Picking => "Choosing files…".into(),
            DropzoneState::DragOver {
                valid: kivact_interaction::Validity::Invalid(message),
            } => message,
            _ => String::new(),
        }),
        |r: &DropzoneTheme| r.status_recipe(),
        variants,
        move || state.get(),
    )
    .node_ref(status_ref)
    .with_node(|n| n.role(Role::Status))
}

/// Render the independent accessible label and application guidance recipes.
fn captions(
    label: Value<String>,
    hint: Value<String>,
    state: Value<DropzoneState>,
    variants: &kivact_core::view::Variants,
) -> (Node, Node) {
    let label_view = crate::layer::with_recipe_style(
        text(label),
        |r: &DropzoneTheme| r.label_recipe(),
        variants,
        move || state.get(),
    );
    let hint_view = crate::layer::with_recipe_style(
        text(hint),
        |r: &DropzoneTheme| r.hint_recipe(),
        variants,
        move || state.get(),
    );
    (label_view, hint_view)
}

/// Route native drag events through the current acceptance policy and drop authority.
fn file_event(
    api: kivact_interaction::DropzoneApi,
    accept_drop: Value<bool>,
    disabled: Value<bool>,
    event: kivact_core::view::FileEvent,
) -> Handled {
    let state = api.state();
    if event.phase == FilePhase::Leave {
        api.leave();
        return Handled::No;
    }
    if !accept_drop.get_untracked()
        || disabled.get_untracked()
        || matches!(state.get_untracked(), DropzoneState::Picking)
    {
        if event.phase == FilePhase::Drop {
            api.leave();
        }
        return Handled::No;
    }
    match event.phase {
        FilePhase::Enter | FilePhase::Over => api.hover(event.files),
        FilePhase::Leave => api.leave(),
        FilePhase::Drop => api.accept(event.files),
    }
    Handled::Yes
}
