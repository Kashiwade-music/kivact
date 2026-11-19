//! Role: Render themed menus recursively over shared menu level parts.
//!
//! One renderer draws both the ordinary trigger's placement and the context
//! placement at the pointer. The builder draws its default trigger button;
//! `.attach(trigger)` takes the caller's trigger and `.attach_context(target)`
//! the right-click target, both `impl Element + IntoView`. A trigger that
//! has its own role (a menu bar's `MenuItem`, read with
//! `Node::accessible_role`) keeps it after attaching; one without becomes a
//! `Button`. The root surface's side and alignment are `.placement` and
//! `.align`, `.at_least_anchor_width` makes it as wide as the trigger, and
//! submenus are drawn with the `sub_surface` recipe. An
//! [`inset`](kivact_interaction::MenuEntry::inset) item or heading puts the
//! `inset` recipe's box before its content so it lines up with the check and
//! radio items. The leading check box and the trailing submenu box are
//! separate slots of the recipe table, and both get the item's actual
//! `MenuItemState`; the meaning of a mark is never switched by faking the
//! state. A menu bar uses the headless menu parts to coordinate its triggers.

pub(crate) mod look;
mod view;

pub use view::{Menu, menu};
