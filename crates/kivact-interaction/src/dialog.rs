//! Role: Dialog attaches modal, alert, drawer and popover surface semantics.
//!
//! [`use_dialog`] is an unstyled dialog, drawer or popover on a popup
//! (`crate::popup`); its parts are `content`, `title`, `description` and
//! `close_trigger`, with [`PopupState`]. The title's
//! `labelled_by` and the description's `described_by` are bound again in the
//! new branch of every opening. The name, the scrim, the panel, the drawer's
//! edge, the body's scrolling and the enter and exit animations are the
//! caller's style (`Style::enter`, `Style::exit`). A modal or a drawer keeps
//! the focus in and restores it through the engine's modal and the overlay's
//! cleanup, which this module does not repeat; a popover gives its required
//! anchor to the popup and shares its rule of leaving by Tab from both the
//! anchor and the surface.
//!
//! [`DialogApi::content`] checks a popover's anchor when it opens: a
//! missing anchor panics with the located misuse diagnostic (the location
//! of the `content()` call). The initial focus (the given node, or
//! the surface's first focusable one through `focus_first_inside`) is
//! requested after the mount through the core's restore path, which checks
//! the target when the request is sent. A surface dismissal is admitted by
//! the [`DismissOn`] rules (assistive technology always; an alert modal's
//! default refuses an outside press), and an admitted one only writes `false`
//! to the open binding, without passing the reason on. Whatever closes a
//! popover while the focus is inside it, the focus goes back to the anchor
//! through the popup's [`FocusReturn`] (Escape, a selection, assistive
//! technology, an explicit close and an unmount), except that an outside
//! press keeps the pressed destination; a modal restores through the
//! overlay's cleanup.
//!
//! `kivact::ui::dialog` draws the standard dialog, drawer and popover from
//! the same parts with `UiRecipes::dialog`. A domain's movable window keeps
//! only its own placement and content and shares the opening, dismissal and
//! focus rules.

use kivact_core::geometry::Edge;
use std::panic::Location;

use kivact_core::ErrorKind;
use kivact_core::custom::{NodeRefExt, Part, PartState, part, raise};
use kivact_core::state::{Binding, Value, Var, on_mount, var};
use kivact_core::view::{A11yAction, A11yRequest, DismissReason, NodeRef, Role, node_ref};

use super::focus_return::{FocusReturn, FocusReturnTarget, PopupCloseReason, ReasonSet};
use super::popup::{PopupApi, PopupState, use_popup};

/// What a [`DialogApi`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogMode {
    /// A modal dialog in the middle of the window (`Role::Dialog`).
    Modal,
    /// A modal dialog that asks for an answer (`Role::AlertDialog`): an
    /// outside press does not close it unless [`DialogApi::dismiss`] says so.
    AlertModal,
    /// A modal sheet attached to an edge of the window (`Role::Dialog`).
    Drawer(Edge),
    /// A non-modal dialog placed at its anchor (`Role::Dialog`).
    Popover,
}

/// Which requests close a [`DialogApi`]. Assistive technology's DismissOn always
/// does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DismissOn {
    /// An unhandled Escape closes it.
    pub escape: bool,
    /// A pointer press outside it (and outside its anchor) closes it.
    pub outside: bool,
}

impl Default for DismissOn {
    fn default() -> Self {
        Self {
            escape: true,
            outside: true,
        }
    }
}

/// What the setters of a [`DialogApi`] store; read where they apply, so they
/// work in any order.
#[derive(Clone, Copy, Default)]
struct DialogSettings {
    dismiss: Option<DismissOn>,
    initial_focus: Option<NodeRef>,
}

/// Copy handle for a dialog, a drawer or a popover, without a look.
///
/// The caller owns the open state, through the `Binding<bool>` given to
/// [`use_dialog`], and draws everything: the scrim, the panel, the title, the
/// description, the body and the buttons. [`show`](kivact_core::view::show) mounts that
/// content in a fresh owner while the dialog is open. Closing is a write of
/// `false` to the binding, from the dialog or from the caller.
///
/// | kind | role | modal |
/// |---|---|---|
/// | [`DialogMode::Modal`] | `Role::Dialog` | yes |
/// | [`DialogMode::AlertModal`] | `Role::AlertDialog` | yes |
/// | [`DialogMode::Drawer`] | `Role::Dialog` | yes |
/// | [`DialogMode::Popover`] | `Role::Dialog` | no |
///
/// Attach [`content`](Self::content) to the caller's `overlay(..)`: centered
/// for a modal, at an edge for a drawer, at the anchor for a popover (the
/// node with the [`trigger`](Self::trigger) part, or the
/// [`anchor`](Self::anchor)). A [`trigger`](Self::trigger) part makes any
/// button open it. The surface is `labelled_by` the node with the
/// [`title`](Self::title) part, a level 2 `Role::Heading`, and `described_by`
/// the node with the [`description`](Self::description) part, when they are
/// mounted. The overlay's accessible name stays the caller's `.name(..)`. A
/// [`close_trigger`](Self::close_trigger) part makes any button close it.
///
/// Escape, an outside press and assistive technology's Dismiss close it, as
/// [`dismiss`](Self::dismiss) allows. While a modal is open, focus, Tab and
/// input stay inside it, and once it closes the focus goes back to the node
/// that had it when it opened: the engine and the overlay do that, and the
/// dialog adds nothing to it. The focus starts on the
/// [`initial_focus`](Self::initial_focus), or else on the first focusable
/// node of the surface. A popover is a [`PopupApi`]: it also closes when the
/// focus leaves both it and its anchor, and the focus goes back to the anchor
/// whenever it closes while the focus is inside, except after an outside
/// press, which keeps the pressed destination.
///
/// ```no_run
/// use kivact_core::{state::signal, view::{NodeBuilder, column, node, text, Placement, Role, node_ref, overlay, show}};
/// use kivact_interaction::{DialogMode, use_dialog};
/// use kivact_core::view::NodeBuilder;
/// let open = signal(false);
/// let confirm = use_dialog(open, DialogMode::AlertModal);
/// let view = show(confirm.is_open(), move || {
///     overlay(node_ref(), Placement::Center, column((
///         text("Delete file").part(confirm.title()),
///         text("This cannot be undone.").part(confirm.description()),
///         node().part(confirm.close_trigger()).name("Cancel").child(text("Cancel")),
///     )))
///     .part(confirm.content())
///     .name("Delete file")
/// });
/// ```
#[derive(Clone, Copy)]
pub struct DialogApi {
    popup: PopupApi,
    content_state: kivact_core::state::Signal<Option<Value<PopupState>>>,
    kind: DialogMode,
    settings: Var<DialogSettings>,
    /// Whether a popover has its anchor: a [`trigger`](Self::trigger) part
    /// or an [`anchor`](Self::anchor).
    anchored: Var<bool>,
    title: NodeRef,
    description: NodeRef,
}

/// Create a dialog of `kind`, shown while `open` is true. Setup only.
#[track_caller]
pub fn use_dialog(open: impl kivact_core::state::IntoBinding<bool>, kind: DialogMode) -> DialogApi {
    DialogApi {
        popup: use_popup(open),
        content_state: kivact_core::state::signal(None),
        kind,
        settings: var(DialogSettings::default()),
        anchored: var(false),
        title: node_ref(),
        description: node_ref(),
    }
}

impl DialogApi {
    /// Drawing state of the mounted trigger.
    pub fn trigger_state(self) -> Value<PopupState> {
        self.popup.trigger_state()
    }
    /// Drawing state of the mounted dialog surface.
    pub fn content_state(self) -> Value<PopupState> {
        kivact_core::state::memo(move || {
            self.content_state
                .get()
                .map(|state| state.get())
                .unwrap_or(PopupState {
                    input: PartState::default(),
                    open: self.is_open().get(),
                })
        })
    }

    /// Put the focus on this node when the dialog opens, instead of the first
    /// focusable one. The content has to mount the node.
    pub fn initial_focus(self, target: NodeRef) -> Self {
        if self.popover() {
            self.popup.initial_focus(target);
        } else {
            self.settings
                .update(|settings| settings.initial_focus = Some(target));
        }
        self
    }

    /// Choose which requests close it. The default is Escape and an outside
    /// press; for [`DialogMode::AlertModal`] it is Escape only.
    pub fn dismiss(self, dismiss: DismissOn) -> Self {
        self.settings
            .update(|settings| settings.dismiss = Some(dismiss));
        self
    }

    /// The node a [`DialogMode::Popover`] is placed at, for a popover opened
    /// by a node without the [`trigger`](Self::trigger) part. A popover needs
    /// an anchor or a trigger; the other kinds ignore it. Setup only.
    pub fn anchor(self, anchor: NodeRef) -> Self {
        self.popup.anchor(anchor);
        self.anchored.set(true);
        self
    }

    /// The button that opens the dialog: activation opens a closed dialog
    /// and closes an open popover, and it publishes `expanded` and
    /// `controls`. It is a popover's anchor: the popover is placed at it and
    /// the focus goes back to it. Setup only.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        self.anchored.set(true);
        self.popup.trigger()
    }

    /// The application's open state.
    pub fn is_open(self) -> Value<bool> {
        self.popup.is_open()
    }

    fn popover(self) -> bool {
        self.kind == DialogMode::Popover
    }

    fn rules(self) -> DismissOn {
        self.settings
            .with(|settings| settings.dismiss)
            .unwrap_or(match self.kind {
                DialogMode::AlertModal => DismissOn {
                    outside: false,
                    ..DismissOn::default()
                },
                _ => DismissOn::default(),
            })
    }

    fn close(self, reason: PopupCloseReason) {
        if self.popover() {
            self.popup.close(reason);
        } else {
            self.popup.open_binding().set(false);
        }
    }

    fn state(self) -> impl Fn(PartState) -> PopupState + 'static {
        let open = self.is_open();
        move |input| PopupState {
            input,
            open: open.get(),
        }
    }

    /// The floating surface, for the caller's `overlay(..)`.
    ///
    /// It sets the role, modality, `labelled_by` and `described_by`, connects
    /// the overlay's dismissal (after the caller's own dismissal handlers) and
    /// requests the initial focus on each opening. A popover without a
    /// trigger or an anchor is a usage error when it opens.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        let at = Location::caller();
        let allowed = move |reason: DismissReason| match reason {
            DismissReason::Escape => self.rules().escape,
            DismissReason::OutsidePress => {
                // An outside press keeps the pressed destination, accepted or
                // not: for the rest of this opening a popover returns the
                // focus to its anchor only for a dismissal. The next opening
                // installs the full policy again.
                if self.popover() {
                    self.popup
                        .set_opening_policy(FocusReturn::anchor(self.popup.anchor_ref()));
                }
                self.rules().outside
            }
            DismissReason::Accessibility => true,
            DismissReason::Timeout => false,
        };
        let open = self.popup.open_binding();
        let surface = if self.popover() {
            self.popup.content_with(allowed)
        } else {
            let input = part(self.state());
            Part::new(input.state(), move |host| {
                host.part(input)
                    .node_ref(self.popup.content_ref())
                    .a11y_action(A11yAction::Dismiss, move |event| {
                        let A11yRequest::Dismiss(reason) = event.request else {
                            unreachable!(
                                "invariant: a dismissal action receives a dismissal request"
                            )
                        };
                        if allowed(reason) {
                            open.set(false);
                        }
                    })
            })
        };
        let part_state = surface.state();
        self.content_state.set(Some(part_state));
        Part::new(part_state, move |host| {
            let surface = host.part(surface);
            let surface = if self.popover() {
                surface
            } else {
                surface.modal()
            };
            surface
                .role(match self.kind {
                    DialogMode::AlertModal => Role::AlertDialog,
                    _ => Role::Dialog,
                })
                .labelled_by(self.title)
                .described_by(self.description)
                .on_attach(move |mount| self.bind(mount.node, at))
        })
    }

    /// Per-opening focus: the start and, for a popover, the return. Setup
    /// only, under the mounted surface's owner.
    fn bind(self, surface: NodeRef, at: &'static Location<'static>) {
        if self.popover() && !self.anchored.get() {
            raise(
                ErrorKind::Message,
                "DialogApi: a popover needs an anchor (DialogApi::trigger or DialogApi::anchor)",
            );
        }
        if self.popover() {
            // Whatever closes a popover while the focus is inside it, the
            // focus goes back to the anchor. A modal gets this from the
            // overlay's own cleanup.
            self.popup.set_opening_policy(FocusReturn {
                target: FocusReturnTarget::Anchor(self.popup.anchor_ref()),
                on: ReasonSet::DISMISS
                    .union(ReasonSet::PROGRAMMATIC)
                    .union(ReasonSet::UNMOUNT),
            });
        }
        match self.settings.with(|settings| settings.initial_focus) {
            Some(target) if !self.popover() => on_mount(move || {
                if target.mounted_node().is_some() {
                    target.focus();
                }
            }),
            Some(_) => {}
            None => on_mount(move || surface.focus_first_inside()),
        }
    }

    /// The title: a level 2 heading the surface is `labelled_by`.
    #[track_caller]
    pub fn title(self) -> Part<PopupState> {
        let input = part(self.state());
        let part_state = input.state();
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .node_ref(self.title)
                .role(Role::Heading)
                .level(2_u32)
        })
    }

    /// A description the surface is `described_by`.
    #[track_caller]
    pub fn description(self) -> Part<PopupState> {
        let input = part(self.state());
        let part_state = input.state();
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input).node_ref(target).node_ref(self.description)
        })
    }

    /// A button whose activation closes the dialog.
    ///
    /// A popover's focus returns to its anchor; a modal's to the node that had
    /// it when it opened.
    #[track_caller]
    pub fn close_trigger(self) -> Part<PopupState> {
        let input = part(self.state());
        let part_state = input.state();
        let target = kivact_core::view::node_ref();
        Part::new(part_state, move |host| {
            host.part(input)
                .node_ref(target)
                .role(Role::Button)
                .focusable(true)
                .on_press(move || self.close(PopupCloseReason::Selection))
        })
    }
}
