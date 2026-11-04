//! Role: Root component mounting and lifecycle.
//!
//! The App's root: the root owner and what lives under it for the whole
//! App (the host environment signals, the keyboard focus source, the
//! theme), the root node and the App's view mounted below it.
//!
//! Invariants:
//! - Started once per App, when the host starts: the client creates the
//!   root owner (recorded in the runtime so that it disposes it first at
//!   shutdown), runtime construction installs the complete view state, then
//!   [`mount_root`] mounts the App's view below it.
//! - The root node resolves its style on the one style path of every node
//!   (`mount::bind_style`): it fills the window with the theme's
//!   surface and carries the theme's text defaults.
//! - The root node has no creation location (diagnostics show the nearest
//!   located node instead).

use std::panic::Location;
use std::rc::Rc;

use crate::i18n::{Locale, Messages};
use kivact_engine::contract::NodeType;
use kivact_runtime::{IntoValue, Owner, Runtime, Signal, Value};
use kivact_style::Theme;

use crate::view::boundary::mount_in;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::View;
use crate::view::mount::{NodeStyle, bind_style};
use crate::view::state::ViewState;

/// Fill the root owner `owner` of `rt`'s App with the `settings` (theme,
/// locale and messages, each a signal the root provides; the locale is the
/// host's, `en-US` if it names none, unless the App sets one), style the
/// root node and mount the view `build` returns below it (set up in the
/// root owner). `at` is where the App was started: diagnostics name the
/// root owner by it.
pub(crate) fn mount_root(
    rt: &Rc<Runtime>,
    owner: Owner,
    settings: crate::launch::AppOptions,
    build: Box<dyn FnOnce() -> View>,
    at: &'static Location<'static>,
) {
    let (theme, app_locale, app_messages) = settings.into_view_settings();
    let env = ViewState::of(rt).env();
    // The App's own locale is a constant; without one the locale is the
    // host's, which follows `Environment::locale`.
    let locale: Value<Locale> = match app_locale {
        Some(named) => owner.constant(named, at),
        None => {
            let host = env.field("locale", |e| e.locale.clone());
            owner.with_setup(|| {
                kivact_runtime::memo(move || host.with(|tag| host_locale(tag.as_deref())))
            })
        }
    };
    let theme: Signal<Theme> = owner.signal(theme, at);
    kivact_style::internal::install_theme(owner, theme.into_value());
    crate::view::boundary::install_locale(owner, locale);
    let messages: Signal<Messages> = owner.signal(app_messages, at);
    crate::view::boundary::install_messages(owner, messages.into_value());
    let root = ViewState::tree(rt).root;
    let style = NodeStyle {
        layers: vec![Rc::new(|| {
            kivact_style::style()
                .size(kivact_style::pct(100), kivact_style::pct(100))
                .background(kivact_style::ColorToken::Surface)
        })],
        ..NodeStyle::default()
    };
    bind_style(&Mcx { rt, owner }, root, NodeType::Root, style, at);
    let child = mount_in(rt, owner, build);
    ViewState::tree_mut(rt).insert(root, child, None);
}

/// The locale the host reports (`en-US` when it names none).
pub(crate) fn host_locale(tag: Option<&str>) -> Locale {
    tag.map(Locale::new).unwrap_or_default()
}
