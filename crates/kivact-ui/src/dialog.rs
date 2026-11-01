//! Role: Render themed dialog frames while shared parts own their lifetime.
//!
//! The builder is itself the view: the body comes from `.content(factory)`
//! and the actions row from `.footer(factory)`, and the same renderer draws
//! Modal, AlertModal, Drawer and Popover from `use_dialog`'s parts with
//! [`DialogState`] (`input`, `open`, `kind`, `media`). A popover gets no
//! header close target. The header is a column of the optional media view,
//! the title and the description (its recipe may lay them out as a grid,
//! knowing [`DialogState::media`]); a hidden title leaves the heading out,
//! and a header with nothing left to show is left out. The close button is
//! the panel's last child (its recipe places it, usually at the top right),
//! so the initial focus goes to the content's first control and the close
//! button is the last tab stop. A dialog without `.content(..)` has no
//! body. When only the footer has a close,
//! `closable(false)` leaves the header's target out without changing the
//! dismissal rules. A drawer's body scrolls and its panel is fixed to its
//! edge. Entering and leaving are the `enter` and `exit` styles the surface
//! recipe returns.
use super::DialogState;
use super::layer::with_recipe_style;
use super::recipes::{Recipe, UiRecipes};
use super::variants::{Variant, Variants};
use kivact_core::control::{Node, NodeBuilder, column, node, row, scroll, text};
use kivact_core::primitive::{NodeExt, Part};
use kivact_core::style::{Length, Position, pct, style};
use kivact_core::view::{
    IntoText, IntoValue, IntoView, NodeRef, OverlayAlign, Placement, View, node_ref, overlay,
};
use kivact_core::{Binding, Value};
use kivact_interaction::{DialogKind, Dismiss, PopupState, Side};
use std::rc::Rc;
pub struct Dialog {
    open: Binding<bool>,
    title: Value<String>,
    title_hidden: Value<bool>,
    kind: DialogKind,
    placement: Placement,
    align: Option<OverlayAlign>,
    width: Option<Value<Length>>,
    description: Option<Value<String>>,
    initial_focus: Option<NodeRef>,
    dismiss: Option<Dismiss>,
    closable: Value<bool>,
    media: Option<Rc<dyn Fn() -> View>>,
    content: Option<Rc<dyn Fn() -> View>>,
    footer: Option<Rc<dyn Fn() -> View>>,
    variants: Variants,
}
/// A themed modal, alert, drawer or anchored popover on the shared dialog parts.
///
/// The builder is the view: give it a body with [`content`](Dialog::content)
/// and optional actions with [`footer`](Dialog::footer). Open it by writing
/// `open`, or from a button with [`attach`](Dialog::attach); a
/// [`DialogKind::Popover`] is placed at the button it is attached to and
/// needs one.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact::ui::DialogKind;
///
/// fn rename(open: Signal<bool>) -> impl IntoView {
///     dialog("Rename", open)
///         .content(|| text("Choose a new name."))
///         .footer(move || button("Done").on_press(move || open.set(false)))
/// }
///
/// fn help() -> impl IntoView {
///     let open = signal(false);
///     dialog("Help", open)
///         .kind(DialogKind::Popover)
///         .content(|| text("Press Tab to move between fields."))
///         .attach(button("Help"))
/// }
/// ```
#[track_caller]
pub fn dialog(title: impl IntoText, open: impl Into<Binding<bool>>) -> Dialog {
    Dialog {
        open: open.into(),
        title: title.into_text_value(),
        title_hidden: false.into_value(),
        kind: DialogKind::Modal,
        placement: Placement::Below,
        align: None,
        width: None,
        description: None,
        initial_focus: None,
        dismiss: None,
        closable: true.into_value(),
        media: None,
        content: None,
        footer: None,
        variants: Variants::default(),
    }
}
impl Dialog {
    /// Give the component a [`Variant`]: a look beyond the meanings its
    /// state carries, which every recipe of the component reads with
    /// [`Look::variant`](crate::Look::variant). A later value of the same
    /// type replaces the earlier one; a theme that does not know the type
    /// draws the plain look.
    ///
    /// The value is a constant: the recipes read the values given before
    /// the component is mounted, and a value never changes afterwards. A
    /// look that changes while the component lives is a state the recipe
    /// reads, or a reactive `.style(..)` of the caller's.
    pub fn variant(self, v: impl Variant) -> Self {
        self.variants.add(v);
        self
    }
}
/// What every part of one dialog tells its recipes besides its own state.
#[derive(Clone, Copy)]
struct Shape {
    kind: DialogKind,
    media: bool,
}
impl Shape {
    fn state(self, state: PopupState) -> DialogState {
        DialogState {
            input: state.input,
            open: state.open,
            kind: self.kind,
            media: self.media,
        }
    }
}
fn themed(
    root: Node,
    part: Part<PopupState>,
    shape: Shape,
    variants: &Variants,
    pick: fn(&UiRecipes) -> &Recipe<DialogState>,
) -> Node {
    let state = part.state();
    with_recipe_style(root.part(part), pick, variants, move || {
        shape.state(state.get())
    })
}
fn wrapper(
    root: Node,
    open: Value<bool>,
    shape: Shape,
    variants: &Variants,
    pick: fn(&UiRecipes) -> &Recipe<DialogState>,
) -> Node {
    let part = kivact_core::primitive::part(move |input| PopupState {
        input,
        open: open.get(),
    });
    themed(root, part, shape, variants, pick)
}
impl Dialog {
    /// Override the theme recipe width with a constant or reactive length.
    ///
    /// The default theme uses 480 logical pixels. The panel keeps its theme's
    /// maximum-width constraint when this value changes.
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = Some(width.into_value());
        self
    }

    /// Include the header close action while true (the default), constant or
    /// reactive; dismissal policy remains independent.
    #[track_caller]
    pub fn closable(mut self, closable: impl IntoValue<bool>) -> Self {
        self.closable = closable.into_value();
        self
    }

    /// Keep the title as the dialog's accessible name while leaving the
    /// visible heading out (while true; constant or reactive), for a panel
    /// whose content says what it is, such as a popover that holds a
    /// calendar. A header without a description or a media view is then
    /// left out too.
    #[track_caller]
    pub fn hide_title(mut self, hide: impl IntoValue<bool>) -> Self {
        self.title_hidden = hide.into_value();
        self
    }

    /// A view at the head of the header, before the title: an icon or an
    /// image that says what the dialog is about (shadcn's
    /// `AlertDialogMedia`). It is built each time the dialog opens, in a box
    /// the `media` recipe styles, and the header's recipes know of it from
    /// [`DialogState::media`].
    pub fn media<V: IntoView>(mut self, media: impl Fn() -> V + 'static) -> Self {
        self.media = Some(Rc::new(move || media().into_view()));
        self
    }

    /// The body, built in a fresh owner each time the dialog opens.
    pub fn content<V: IntoView>(mut self, body: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Rc::new(move || body().into_view()));
        self
    }

    /// The actions row under the body, built each time the dialog opens; a
    /// footer that builds nothing is left out.
    pub fn footer<V: IntoView>(mut self, footer: impl Fn() -> V + 'static) -> Self {
        self.footer = Some(Rc::new(move || footer().into_view()));
        self
    }

    pub fn kind(mut self, kind: DialogKind) -> Self {
        self.kind = kind;
        self
    }

    /// The side of its trigger a [`DialogKind::Popover`] opens on (default
    /// [`Placement::Below`]); the host flips it to the opposite side when
    /// the panel does not fit. The other kinds are centred or fixed to their
    /// edge and ignore it. A constant: it is the shape of the layout.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }

    /// Where along its trigger's side a [`DialogKind::Popover`] sits (the
    /// overlay's [`align`](kivact_core::view::Overlay::align); its start by
    /// default). The other kinds ignore it. A constant, as the placement.
    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.align = Some(align);
        self
    }

    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }
    pub fn initial_focus(mut self, target: NodeRef) -> Self {
        self.initial_focus = Some(target);
        self
    }
    pub fn dismiss(mut self, dismiss: Dismiss) -> Self {
        self.dismiss = Some(dismiss);
        self
    }

    /// Open the dialog from `trigger` (a press, Enter or Space; a popover's
    /// trigger also closes it) and return the trigger with the dialog. The
    /// trigger publishes `expanded` and `controls`, keeps its own name and
    /// role (a button without a role becomes a `Role::Button`), and is where
    /// a [`DialogKind::Popover`] is placed and where the focus goes back to.
    pub fn attach<T: NodeBuilder + IntoView>(self, trigger: T) -> View {
        self.render(Some(trigger))
    }
}

impl IntoView for Dialog {
    fn into_view(self) -> View {
        self.render(None::<Node>)
    }
}

impl Dialog {
    fn render<T: NodeBuilder + IntoView>(self, trigger: Option<T>) -> View {
        let mut behavior = kivact_interaction::use_dialog(self.open, self.kind);
        if let Some(target) = self.initial_focus {
            behavior = behavior.initial_focus(target);
        }
        if let Some(dismiss) = self.dismiss {
            behavior = behavior.dismiss(dismiss);
        }
        let mut anchor = node_ref();
        let trigger = trigger.map(|mut trigger| {
            let role = trigger.accessible_role();
            let part = behavior.trigger();
            anchor = part.node_ref();
            let trigger = trigger.part(part);
            match role {
                Some(role) => trigger.role(role),
                None => trigger,
            }
            .into_view()
        });
        let kind = self.kind;
        let shape = Shape {
            kind,
            media: self.media.is_some(),
        };
        let open = self.open.into_value();
        let title = self.title;
        let hidden = self.title_hidden;
        let variants = self.variants.clone();
        let dialog = kivact_core::view::show(behavior.shown(), move || {
            let variants = &variants;
            let heading = {
                let variants = variants.clone();
                move || {
                    themed(text(title), behavior.title(), shape, &variants, |r| {
                        r.dialog_recipes().title_recipe()
                    })
                }
            };
            let closable = self.closable;
            let close = (kind != DialogKind::Popover).then(|| {
                let variants = variants.clone();
                kivact_core::view::show(closable, move || {
                    themed(
                        node()
                            .name(kivact_core::i18n::message("dialog.close", "Close"))
                            .child(kivact_core::control::icon("x")),
                        behavior.close_trigger(),
                        shape,
                        &variants,
                        |r| r.dialog_recipes().close_recipe(),
                    )
                })
            });
            let description = self.description.map(|label| {
                themed(text(label), behavior.description(), shape, variants, |r| {
                    r.dialog_recipes().description_recipe()
                })
            });
            let media = self.media.as_ref().map(|media| {
                wrapper(node().child(media()), open, shape, variants, |r| {
                    r.dialog_recipes().media_recipe()
                })
            });
            // The media, the title and the description form the header; the
            // close button comes last, so the content's first control takes
            // the initial focus and the close button is the last tab stop
            // (its recipe places it, usually at the top right). A header of
            // the title alone goes with a hidden title.
            let header = if description.is_some() || media.is_some() {
                let heading = kivact_core::view::show(move || !hidden.get(), heading);
                wrapper(
                    column((media, heading, description)),
                    open,
                    shape,
                    variants,
                    |r| r.dialog_recipes().header_recipe(),
                )
                .into_view()
            } else {
                let variants = variants.clone();
                kivact_core::view::show(
                    move || !hidden.get(),
                    move || {
                        wrapper(column(heading()), open, shape, &variants, |r| {
                            r.dialog_recipes().header_recipe()
                        })
                    },
                )
                .into_view()
            };
            // A dialog without content has no body, so the surface's gap
            // does not open twice between the header and the footer.
            let content = self.content.as_ref().map(|body| {
                let content = wrapper(node().child(body()), open, shape, variants, |r| {
                    r.dialog_recipes().body_recipe()
                });
                if matches!(kind, DialogKind::Drawer(_)) {
                    scroll(content)
                        .style(style().grow(1).min_height(0))
                        .into_view()
                } else {
                    content.into_view()
                }
            });
            let footer = self
                .footer
                .as_ref()
                .map(|footer| footer())
                .filter(|footer| !kivact_core::primitive::view_is_empty(footer))
                .map(|footer| {
                    wrapper(row(footer), open, shape, variants, |r| {
                        r.dialog_recipes().footer_recipe()
                    })
                });
            let surface = behavior.content();
            let surface_state = surface.state();
            let mut panel = with_recipe_style(
                column((header, content, footer, close)),
                |r| r.dialog_recipes().surface_recipe(),
                variants,
                move || shape.state(surface_state.get()),
            );
            if let DialogKind::Drawer(side) = kind {
                panel = panel.style(edge(side));
            }
            if let Some(width) = self.width {
                panel = panel.style(move || style().width(width.get()));
            }
            let popover = kind == DialogKind::Popover;
            let layer = overlay(
                anchor,
                if popover {
                    self.placement
                } else {
                    Placement::Center
                },
                panel,
            )
            .part(surface)
            .name(title);
            if popover {
                let layer = match self.align {
                    Some(align) => layer.align(align),
                    None => layer,
                };
                return layer.offset(4.0).into_view();
            }
            (
                wrapper(node(), open, shape, variants, |r| {
                    r.dialog_recipes().scrim_recipe()
                }),
                layer,
            )
                .into_view()
        })
        .into_view();
        (trigger, dialog).into_view()
    }
}
/// The edge a drawer is fixed to. A side drawer's width is its surface
/// recipe's (the default recipe keeps it within the window); a top or
/// bottom one spans the window.
fn edge(side: Side) -> kivact_core::style::Style {
    let fixed = style().position(Position::Fixed);
    match side {
        Side::Left => fixed.left(0).top(0).bottom(0),
        Side::Right => fixed.right(0).top(0).bottom(0),
        Side::Top => fixed.left(0).right(0).top(0).width(pct(100)),
        Side::Bottom => fixed.left(0).right(0).bottom(0).width(pct(100)),
    }
}
