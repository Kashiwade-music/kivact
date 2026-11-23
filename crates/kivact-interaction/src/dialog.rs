//! Role: Dialog attaches modal, alert and drawer surface semantics.
//!
//! [`use_dialog`] is an unstyled modal dialog, alert dialog or drawer on a
//! popup (`crate::popup`); its parts are `content`, `title`, `description` and
//! `close_trigger`, with [`PopupState`]. The title and the description are the
//! popup's, so the surface is `labelled_by` the title and `described_by` the
//! description in the new branch of every opening. The name, the scrim, the
//! panel, the drawer's edge, the body's scrolling and the enter and exit
//! animations are the caller's style (`Style::enter`, `Style::exit`). A modal
//! or a drawer keeps the focus in and restores it through the engine's modal
//! and the overlay's cleanup, which this module does not repeat. The initial
//! focus (the given node, or the surface's first focusable one through
//! `focus_first_inside`) is requested after the mount through the core's
//! restore path, which checks the target when the request is sent. A surface
//! dismissal is admitted by the [`DismissOn`] rules (assistive technology
//! always; an alert modal's default refuses an outside press), and an admitted
//! one only writes `false` to the open binding, without passing the reason on.
//!
//! `kivact::ui::dialog` draws the standard dialog and drawer from the same
//! parts. A domain's movable window keeps only its own placement and content
//! and shares the opening, dismissal and focus rules.

use kivact_core::geometry::Edge;

use kivact_core::custom::{Part, PartState, part};
use kivact_core::state::Value;
use kivact_core::view::{
    DismissReason, Element, IntoView, NodeRef, Placement, Role, View, overlay, show,
};

use crate::options_cell::OptionsCell;

use super::focus_return::{FocusReturn, PopupCloseReason};
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
}

/// Which requests close a [`DialogApi`]. Assistive technology's DismissOn always
/// does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DismissOn {
    /// An unhandled Escape closes it.
    pub escape: bool,
    /// A pointer press outside it closes it.
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
struct DialogOptions {
    dismiss: Option<DismissOn>,
}

/// Copy handle for a dialog or a drawer, without a look.
///
/// The caller owns the open state, through the `Signal<bool>` given to
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
///
/// Attach [`content`](Self::content) to the caller's `overlay(..)`, centred for
/// a modal and at an edge for a drawer, or use [`surface`](Self::surface). A
/// [`trigger`](Self::trigger) part makes any button open it. The surface is
/// `labelled_by` the node with the [`title`](Self::title) part, a level 2
/// `Role::Heading`, and `described_by` the node with the
/// [`description`](Self::description) part, when they are mounted. The
/// overlay's accessible name stays the caller's `.name(..)`. A
/// [`close_trigger`](Self::close_trigger) part makes any button close it.
///
/// Escape, an outside press and assistive technology's Dismiss close it, as
/// [`dismiss`](Self::dismiss) allows. While a modal is open, focus, Tab and
/// input stay inside it, and once it closes the focus goes back to the node
/// that had it when it opened: the engine and the overlay do that, and the
/// dialog adds nothing to it. The focus starts on the
/// [`initial_focus`](Self::initial_focus), or else on the first focusable
/// node of the surface.
///
/// ```no_run
/// use kivact::{state::signal, view::{Element, column, node, text, Placement, Role, node_ref, overlay, show}};
/// use kivact::interaction::{DialogMode, use_dialog};
/// let open = signal(false);
/// let confirm = use_dialog(open, DialogMode::AlertModal);
/// let view = show(confirm.is_open(), move || {
///     overlay(node_ref(), Placement::Center, column((
///         text("Delete file").part(confirm.title()),
///         text("This cannot be undone.").part(confirm.description()),
///         node().part(confirm.close_trigger()).name("Cancel").child(text("Cancel")),
///     )))
///     .with_node(|node| node.part(confirm.content()))
///     .name("Delete file")
/// });
/// ```
#[derive(Clone, Copy)]
pub struct DialogApi {
    popup: PopupApi,
    mode: DialogMode,
    settings: OptionsCell<DialogOptions>,
}

/// Create a dialog of `kind`, shown while `open` is true. Setup only.
#[track_caller]
pub fn use_dialog(open: kivact_core::state::Signal<bool>, kind: DialogMode) -> DialogApi {
    DialogApi {
        popup: use_popup(open).initial_focus_first(),
        mode: kind,
        settings: OptionsCell::new(DialogOptions::default()),
    }
}

impl DialogApi {
    /// Put the focus on this node when the dialog opens, instead of the first
    /// focusable one. The content has to mount the node.
    pub fn initial_focus(self, target: NodeRef) -> Self {
        self.popup.initial_focus_restore(target);
        self
    }

    /// Choose which requests close it. The default is Escape and an outside
    /// press; for [`DialogMode::AlertModal`] it is Escape only.
    pub fn dismiss(self, dismiss: DismissOn) -> Self {
        self.settings
            .update(|settings| settings.dismiss = Some(dismiss));
        self
    }

    /// Snapshot this target once per opening for keyboard, selection and
    /// programmatic focus restoration. Outside presses retain outside focus.
    /// Setup only; the supplied value is read by the opening's focus owner.
    pub fn return_focus(self, target: impl kivact_core::state::IntoValue<NodeRef>) -> Self {
        self.popup
            .focus_return(FocusReturn::anchor_value(target.into_value()));
        self
    }

    /// The button that opens the dialog: activation opens a closed dialog,
    /// and it publishes `expanded` and `controls`. Setup only.
    #[track_caller]
    pub fn trigger(self) -> Part<PopupState> {
        self.popup.trigger()
    }

    /// The application's open state.
    pub fn is_open(self) -> Value<bool> {
        self.popup.is_open()
    }

    /// Show the dialog unless it is shown.
    pub fn show(self) {
        self.popup.show();
    }

    /// Hide the dialog unless it is hidden.
    pub fn hide(self) {
        if self.is_open().get_untracked() {
            self.popup.open_binding().set(false);
        }
    }

    /// Show a hidden dialog, hide a shown one.
    pub fn toggle(self) {
        if self.is_open().get_untracked() {
            self.hide();
        } else {
            self.show();
        }
    }

    fn rules(self) -> DismissOn {
        self.settings.get().dismiss.unwrap_or(match self.mode {
            DialogMode::AlertModal => DismissOn {
                outside: false,
                ..DismissOn::default()
            },
            _ => DismissOn::default(),
        })
    }

    fn close(self, _reason: PopupCloseReason) {
        self.popup.open_binding().set(false);
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
    /// It sets the role and modality, connects the overlay's dismissal (after
    /// the caller's own dismissal handlers) and requests the initial focus on
    /// each opening. The popup sets the `labelled_by` and `described_by`.
    #[track_caller]
    pub fn content(self) -> Part<PopupState> {
        let allowed = move |reason: DismissReason| match reason {
            DismissReason::Escape | DismissReason::WindowClose => self.rules().escape,
            DismissReason::OutsidePress => self.rules().outside,
            DismissReason::Accessibility => true,
            DismissReason::Timeout => false,
        };
        let surface = self.popup.content_with(allowed);
        let part_state = surface.state();
        Part::new(part_state, move |host| {
            host.part(surface).modal().role(match self.mode {
                DialogMode::AlertModal => Role::AlertDialog,
                _ => Role::Dialog,
            })
        })
    }

    /// The title: a level 2 heading the surface is `labelled_by`.
    #[track_caller]
    pub fn title(self) -> Part<PopupState> {
        self.popup.title()
    }

    /// A description the surface is `described_by`.
    #[track_caller]
    pub fn description(self) -> Part<PopupState> {
        self.popup.description()
    }

    /// A button whose activation closes the dialog.
    ///
    /// A modal's focus returns to the node that had it when it opened.
    #[track_caller]
    pub fn close_trigger(self) -> Part<PopupState> {
        let input = part(self.state());
        let part_state = input.state();
        Part::new(part_state, move |host| {
            host.part(input)
                .role(Role::Button)
                .focusable(true)
                .on_press(move || self.close(PopupCloseReason::Selection))
        })
    }

    /// The node the surface is placed at.
    pub(crate) fn anchor_ref(self) -> NodeRef {
        self.popup.anchor_ref()
    }

    /// The floating surface placed at the anchor, for the caller's
    /// `overlay(..)`: mounted while the dialog is shown.
    #[track_caller]
    pub fn surface<V: IntoView>(
        self,
        placement: Placement,
        content: impl Fn() -> V + 'static,
    ) -> View {
        show(self.is_open(), move || {
            overlay(self.anchor_ref(), placement, content())
                .with_node(|node| node.part(self.content()))
        })
        .into_view()
    }
}
