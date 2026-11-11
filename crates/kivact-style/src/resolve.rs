//! Role: Style property resolution against the theme into node props.
//!
//! Style + theme → the engine's node props (`wire`): layout, text and
//! paint (backgrounds, borders, radii, shadows, effects, the transform and
//! the z-index), and the paint deltas of the interaction states. The
//! values a style holds resolve themselves (`Color::resolve`,
//! `paint::fill_layer`, `effect::filter_ops`, `text::resolve_extras`), so a
//! `Draw` resolves them the same way without a `Style`.
//!
//! Invariants:
//! - Every color position (gradient stops, each border side, shadows, drop
//!   shadows, the mask) resolves its theme token here, once.
//! - A solid background over the whole box is an allocation-free
//!   `Fill::Solid`; only a node that uses effects holds an
//!   `Option<Box<Effects>>`.
//! - A state style is resolved over the base by groups (background,
//!   border, radius, shadow, ring, effects, transform, z-index), so a state
//!   that sets only `border_color` keeps the base width, and one that sets
//!   only `scale` keeps the base rotation.
//! - A transition over the paint properties (`TransitionProp::All`) works the same for a
//!   signal or style change and for the host's hover, pressed and focus
//!   switches: it interpolates the opacity, a solid background, each
//!   border color, the canonical transform and its origin, the first two
//!   shadows and the filter amounts; gradients, images, shapes, the
//!   z-index and the order of filters switch to the target at once.
//! - A dynamic style that drops a state writes empty `StateStyles`, and one
//!   that drops a text property writes `None`, through the typed setter, so
//!   the node goes back to what it inherits.

mod compile;
mod describe;
mod paint;

pub use compile::{CompiledStyle, ForcedStates, Interactive, compile_style};
pub use describe::{describe_state_styles, describe_style};
