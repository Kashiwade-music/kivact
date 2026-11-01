//! Role: Root component mounting and lifecycle.
//!
//! The App's root: the root owner and what lives under it for the whole
//! App (the host environment signals, the keyboard focus source, the
//! theme), the root node and the App's view mounted below it.
//!
//! Invariants:
//! - Mounted once per App, when the host starts (the client creates the
//!   root owner, records it in the runtime so that it disposes it first at
//!   shutdown, and calls [`mount_root`] with it).
//! - The root node resolves its style on the one style path of every node
//!   (`mount::bind_style`): it fills the window with the theme's
//!   surface and carries the theme's text defaults.
//! - The root node has no creation location (diagnostics show the nearest
//!   located node instead).

use std::panic::Location;
use std::rc::Rc;

use crate::i18n::{Locale, Messages};
use kivact_reactive::{IntoValue, Owner, Runtime, Signal, Value};
use kivact_style::Theme;
use kivact_wire::{HostInfo, NodeId, NodeType};

use crate::view::boundary::mount_in;
use crate::view::descriptor::Mcx;
use crate::view::descriptor::View;
use crate::view::mount::{NodeStyle, bind_style};
use crate::view::state::EnvState;
use crate::view::state::ViewState;

/// What the App sets for its whole life (`App::theme`, `App::locale`,
/// `App::messages`).
pub(crate) struct AppSettings {
    pub theme: Theme,
    /// `None`: the App names no locale, so it speaks the host's.
    pub locale: Option<Locale>,
    pub messages: Messages,
}

/// Fill the root owner `owner` of `rt`'s App with the environment `info`
/// reports, the keyboard focus source, the `settings` (theme, locale and
/// messages, each a signal the root provides; the locale is the host's,
/// `en-US` if it names none, unless the App sets one), the root node, and
/// mount the view `build` returns below it (set up in the root owner).
/// `at` is where the App was started: diagnostics name the root owner by
/// it. Returns the root node.
pub(crate) fn mount_root(
    rt: &Rc<Runtime>,
    owner: Owner,
    info: &HostInfo,
    settings: AppSettings,
    build: Option<Box<dyn FnOnce() -> View>>,
    at: &'static Location<'static>,
) -> NodeId {
    let env = Rc::new(EnvState::new(owner, info.env.clone(), at));
    // The App's own locale is a constant; without one the locale is the
    // host's, which follows `HostEnv::locale`.
    let locale: Value<Locale> = match settings.locale.clone() {
        Some(named) => owner.constant(named, at),
        None => {
            let host = env.field("locale", |e| e.locale.clone());
            owner.with_setup(|| {
                kivact_reactive::memo(move || host.with(|tag| host_locale(tag.as_deref())))
            })
        }
    };
    ViewState::of(rt).set_env(env.clone());
    ViewState::of(rt).set_capabilities(info.capabilities);
    crate::view::focus::install(rt, owner, at);
    kivact_motion::MotionState::of(rt).attach_env(
        env.field("reduced_motion", |e| e.reduced_motion),
        env.field("surface_active", |e| e.surface_active),
    );
    let theme: Signal<Theme> = owner.signal(settings.theme, at);
    Theme::install(owner, theme.into_value());
    kivact_i18n::install_locale(owner, locale);
    let messages: Signal<Messages> = owner.signal(settings.messages, at);
    kivact_i18n::install_messages(owner, messages.into_value());
    let root = ViewState::tree_mut(rt).create(Some(NodeType::Root), owner);
    ViewState::tree_mut(rt).root = Some(root);
    let style = NodeStyle {
        layers: vec![Rc::new(|| {
            kivact_style::style()
                .size(kivact_style::pct(100), kivact_style::pct(100))
                .background(kivact_style::ColorToken::Surface)
        })],
        ..NodeStyle::default()
    };
    bind_style(&Mcx { rt, owner }, root, NodeType::Root, style, at);
    if let Some(build) = build {
        let child = mount_in(rt, owner, build);
        ViewState::tree_mut(rt).insert(root, child, None);
    }
    root
}

/// The locale the host reports (`en-US` when it names none).
pub(crate) fn host_locale(tag: Option<&str>) -> Locale {
    tag.map(Locale::new).unwrap_or_default()
}
