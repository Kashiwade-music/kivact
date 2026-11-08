//! Role: Root component mounting and lifecycle.
//!
//! The App's root: the root owner and what lives under it for the whole
//! App (the host environment signals, the keyboard focus source, the
//! theme), the root node and the App's view mounted below it.
//!
//! Invariants:
//! - Started once per App, when the host starts: the client creates the
//!   root owner (recorded in the runtime so that it disposes it first at
//!   shutdown), runtime construction installs the complete view state,
//!   [`install_app_contexts`] installs the App-wide contexts and
//!   [`mount_window_root`] mounts each window's view below its root.
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
use kivact_types::WindowId;

use crate::view::boundary::mount_in;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::View;
use crate::view::mount::{NodeStyle, bind_style};
use crate::view::state::ViewState;

/// Fill the root owner `owner` of `rt`'s App with the `settings` (theme,
/// locale and messages, each a signal the root provides; the locale is the
/// host's, `en-US` if it names none, unless the App sets one). `at` is where
/// the App was started: diagnostics name the root owner by it.
pub(crate) fn install_app_contexts(
    rt: &Rc<Runtime>,
    owner: Owner,
    settings: (
        kivact_runtime::Prop<kivact_style::Theme>,
        Option<Locale>,
        Messages,
    ),
    at: &'static Location<'static>,
) {
    let (theme, app_locale, app_messages) = settings;
    let env = ViewState::of(rt)
        .env(WindowId::MAIN)
        .expect("invariant: the main window record exists before mounting");
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
    // The root publishes a theme source even for a fixed startup value, so
    // style dependencies retain the same theme identity in diagnostics.
    // Reactive inputs keep their existing source and update semantics.
    let theme = match theme.fixed() {
        Some(theme) => owner.signal(theme, at).into_value(),
        None => owner.with_setup(|| theme.into_value(owner, at)),
    };
    kivact_style::internal::install_theme(owner, theme);
    crate::view::boundary::install_locale(owner, locale);
    let messages: Signal<Messages> = owner.signal(app_messages, at);
    crate::view::boundary::install_messages(owner, messages.into_value());
}

/// Style the root node of `window` and mount the view `build` returns below
/// it (set up in `owner`). Returns the window's root node.
pub(crate) fn mount_window_root(
    rt: &Rc<Runtime>,
    window: WindowId,
    owner: Owner,
    build: Box<dyn FnOnce() -> View>,
    at: &'static Location<'static>,
) -> kivact_types::NodeId {
    let root = match ViewState::tree(rt).root(window) {
        Some(root) => root,
        None => ViewState::tree_mut(rt).create_root(window, owner),
    };
    let style = NodeStyle {
        layers: vec![crate::view::element_data::StyleLayer {
            f: Rc::new(|| {
                kivact_style::style()
                    .size(kivact_style::pct(100), kivact_style::pct(100))
                    .background(kivact_style::ColorToken::Surface)
            }),
            at: Some(at),
            origin: crate::view::element_data::LayerOrigin::Base,
        }],
        ..NodeStyle::default()
    };
    bind_style(&Mcx { rt, owner, window }, root, NodeType::Root, style, at);
    let child = mount_in(rt, owner, window, build);
    ViewState::tree(rt).engine().insert(root, child, None);
    root
}

/// The locale the host reports (`en-US` when it names none).
pub(crate) fn host_locale(tag: Option<&str>) -> Locale {
    tag.map(Locale::new).unwrap_or_default()
}
