//! Role: The width class of an alert dialog, which the kit's dialog recipes read.

/// The width class of an alert dialog (alert-dialog.tsx's `data-size`),
/// given to the standard dialog with `.variant(..)`.
///
/// An alert dialog is `kivact::ui::dialog` of kind
/// `DialogMode::AlertModal`: Escape and its buttons close it, a press
/// outside does not. Its cancel and action buttons are the footer, the
/// first one taking the initial focus; `.closable(false)` leaves out the
/// close button, which alert-dialog.tsx does not have, and `.media(..)`
/// puts a 40 px muted square beside or above the title.
///
/// `Default` is `max-w-sm` (384 px) with left-aligned text; `Sm` is
/// `max-w-xs` (320 px) with centered text and a two-column footer.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::{ButtonVariant};
/// use kivact::interaction::DialogMode;
/// use kivact_ui_basic as basic;
///
/// fn confirm(open: Signal<bool>) -> impl IntoView {
///     dialog("Are you absolutely sure?", open)
///         .mode(DialogMode::AlertModal)
///         .variant(basic::AlertDialogSize::Sm)
///         .closable(false)
///         .description("This action cannot be undone.")
///         .footer(move || {
///             (
///                 button("Cancel")
///                     .variant(ButtonVariant::Outline)
///                     .on_press(move || open.set(false)),
///                 button("Continue").on_press(move || open.set(false)),
///             )
///         })
/// }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlertDialogSize {
    /// 384 px wide; the media sits beside the text and the footer is right-aligned.
    #[default]
    Default,
    /// 320 px wide; everything is centered and the two buttons share the footer.
    Sm,
}

impl kivact::ui::Variant for AlertDialogSize {}
