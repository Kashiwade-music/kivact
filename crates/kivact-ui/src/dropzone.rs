//! Role: Render a native file drop target with validation and optional picking.
use kivact_core::platform::{FileDialog, pick_file, pick_files};
use kivact_core::state::{IntoText, IntoValue, Value};
use kivact_core::task::spawn;
use kivact_core::view::{
    FilePhase, Handled, HandlerResult, IntoView, Node, NodeBuilder, Role, View, node, text,
};
use kivact_interaction::{DropzoneState, FileInfo, use_dropzone};
use std::rc::Rc;
type FileValidator = Rc<dyn Fn(&FileInfo) -> Option<String>>;
// xtask-allow(state-fields): Independent setup-only NodeBuilder options; drag and picker lifecycle live in DropzoneState.
/// A validated file drop target. Native file metadata stays application-owned.
pub struct Dropzone {
    root: Node,
    variants: crate::variants::Variants,
    icon: Option<View>,
    label: Value<String>,
    hint: Value<String>,
    filters: Value<Vec<(String, Vec<String>)>>,
    multiple: bool,
    accept_drop: Value<bool>,
    pick_on_press: bool,
    validate: FileValidator,
    on_files: Rc<dyn Fn(Vec<FileInfo>) -> kivact_core::Result<()>>,
}
/// Create a file drop target labelled by `label`.
pub fn dropzone(label: impl IntoText) -> Dropzone {
    Dropzone {
        variants: crate::variants::Variants::default(),
        root: node(),
        icon: None,
        label: label.into_text_value(),
        hint: String::new().into_value(),
        filters: Vec::new().into_value(),
        multiple: false,
        accept_drop: true.into_value(),
        pick_on_press: false,
        validate: Rc::new(|_| None),
        on_files: Rc::new(|_| Ok(())),
    }
}
impl Dropzone {
    /// Accept drag/drop input independently of the native picker. A policy
    /// change to false ends the active drag-over visit without clearing files.
    pub fn accept_drop(mut self, enabled: impl IntoValue<bool>) -> Self {
        self.accept_drop = enabled.into_value();
        self
    }

    /// Add decorative content above the label, mounted once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Add a setup-only visual variant for every component recipe.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }

    /// Receive valid files. Callback errors appear in the drop target.
    pub fn on_files<R: HandlerResult>(mut self, f: impl Fn(Vec<FileInfo>) -> R + 'static) -> Self {
        self.on_files = Rc::new(move |files| f(files).into_outcome().map(|_| ()));
        self
    }
    /// Reject a file with a user-facing error message.
    pub fn validate(mut self, f: impl Fn(&FileInfo) -> Option<String> + 'static) -> Self {
        self.validate = Rc::new(f);
        self
    }
    /// Accept multiple files in one operation.
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }
    /// Open a native picker on pointer, keyboard or assistive activation.
    pub fn pick_on_press(mut self) -> Self {
        self.pick_on_press = true;
        self
    }
    /// Show application-provided guidance.
    pub fn hint(mut self, hint: impl IntoText) -> Self {
        self.hint = hint.into_text_value();
        self
    }
    /// Pass filters to the native picker.
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
impl IntoView for Dropzone {
    fn into_view(self) -> View {
        let Self {
            mut root,
            variants,
            label,
            icon,
            hint,
            filters,
            multiple,
            accept_drop,
            pick_on_press,
            validate,
            on_files,
        } = self;
        let disabled = root.accessible_disabled();
        let disabled = kivact_core::state::memo(move || disabled.is_some_and(|d| d.get()));
        let mut api = use_dropzone()
            .validate(move |file| validate(file))
            .on_files(move |files| on_files(files));
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
            root.part(kivact_interaction::use_button().control())
        } else {
            root.with_node(|n| n.role(Role::Group))
        };
        let (label_view, hint_view) = captions(label, hint, state, &variants);
        crate::layer::with_base_recipe_style(
            root,
            |r| r.dropzone_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .node_ref(reference)
        .name(label)
        .invalid(move || matches!(state.get(), DropzoneState::Failed(_)))
        .on_file(move |event| {
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
        })
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
                    |r| r.dropzone_recipes().icon_recipe(),
                    &variants,
                    move || state.get(),
                )
            }),
            label_view,
            hint_view,
            failure_status(state, &variants),
            drag_status(state, &variants, status_ref),
        ))
        .into_view()
    }
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
fn failure_status(state: Value<DropzoneState>, variants: &crate::variants::Variants) -> View {
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
                    |r| r.dropzone_recipes().status_recipe(),
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
    variants: &crate::variants::Variants,
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
        |r| r.dropzone_recipes().status_recipe(),
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
    variants: &crate::variants::Variants,
) -> (Node, Node) {
    let label_view = crate::layer::with_recipe_style(
        text(label),
        |r| r.dropzone_recipes().label_recipe(),
        variants,
        move || state.get(),
    );
    let hint_view = crate::layer::with_recipe_style(
        text(hint),
        |r| r.dropzone_recipes().hint_recipe(),
        variants,
        move || state.get(),
    );
    (label_view, hint_view)
}
