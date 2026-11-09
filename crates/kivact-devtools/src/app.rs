//! Role: The root of the DevTools app: theme, connection states and session screen.

use kivact::prelude::*;

use crate::args::Args;
use crate::connection;
use crate::panels;
use crate::state::{Connection, Panel, Session};
use crate::{status_bar, toolbar};

/// The root of the app: connect to `args.target`, or discover a target.
pub fn root(args: Args) -> impl IntoView {
    let connection = signal(Connection::Searching);
    on_mount(move || match args.target {
        Some(path) => connection::connect(connection, path),
        None => connection::discover(connection, None),
    });
    shell(connection)
}

/// The app with an explicit initial connection, for tests.
pub fn view_with(initial: Connection) -> impl IntoView {
    shell(signal(initial))
}

/// Install the theme and follow the connection state.
fn shell(connection: Signal<Connection>) -> impl IntoView {
    provide_theme(
        || {
            let mode = match kivact::platform::theme_hint().get() {
                kivact::platform::ThemeHint::Dark => ThemeMode::Dark,
                kivact::platform::ThemeHint::Light => ThemeMode::Light,
            };
            let theme = kivact_ui_basic::theme(mode);
            let text = kivact::style::theme::Typography {
                family: Some("Inter, sans-serif".into()),
                ..theme.text().clone()
            };
            theme.with_text(text)
        },
        move || {
            dynamic(
                move || connection.get(),
                move |state| screen(state, connection),
            )
        },
    )
}

/// The screen of one connection state.
fn screen(state: Connection, connection: Signal<Connection>) -> View {
    match state {
        Connection::Searching => connection::searching_view().into_view(),
        Connection::Choosing(targets) => connection::choosing_view(connection, targets).into_view(),
        Connection::Connecting(path) => connection::connecting_view(path).into_view(),
        Connection::Connected(target) => session_view(Session::new(target, connection)).into_view(),
        Connection::Lost {
            path,
            app_name,
            reason,
        } => connection::lost_view(connection, path, app_name, reason).into_view(),
    }
}

/// The toolbar, the panel tabs, the selected panel, the status bar and the popped-out windows.
///
/// The selected panel sits in a column that takes the rest of the window, so that its body can scroll.
fn session_view(session: Session) -> impl IntoView {
    column((
        toolbar::view(session),
        tabs("Panels", panel_items(), session.panel),
        column(dynamic(
            move || session.panel.get(),
            move |panel| panel_view(panel, session),
        ))
        .style(style().grow(1).shrink(1).min_height(0)),
        status_bar::view(session),
        popped_windows(session),
    ))
    .style(style().size(pct(100), pct(100)).min_height(0))
}

/// The tabs of every panel.
fn panel_items() -> Vec<ListItem<Panel>> {
    Panel::ALL
        .iter()
        .map(|panel| ListItem::new(*panel, panel.title()))
        .collect()
}

/// The content of one panel.
fn panel_view(panel: Panel, session: Session) -> View {
    match panel {
        Panel::Elements => panels::elements::view(session).into_view(),
        Panel::Updates => panels::updates::view(session).into_view(),
        Panel::Performance => panels::performance::view(session).into_view(),
        Panel::Accessibility => panels::accessibility::view(session).into_view(),
        Panel::Input => panels::input::view(session).into_view(),
        Panel::Animations => panels::animations::view(session).into_view(),
        Panel::Async => panels::tasks::view(session).into_view(),
        Panel::Theme => panels::theme::view(session).into_view(),
        Panel::Console => panels::console::view(session).into_view(),
        Panel::Emulation => panels::emulation::view(session).into_view(),
        Panel::Review => panels::review::view(session).into_view(),
        Panel::Scripts => panels::scripts::view(session).into_view(),
    }
}

/// One window per panel the user popped out.
fn popped_windows(session: Session) -> impl IntoView {
    for_each(
        session.popped,
        |panel| *panel,
        move |row| {
            let panel = row.get_untracked();
            window(
                panel.title(),
                binding(
                    move || true,
                    move |open: bool| {
                        if !open {
                            session
                                .popped
                                .update(|popped| popped.retain(|p| *p != panel));
                        }
                    },
                ),
            )
            .size(900, 600)
            .content(move || {
                column((panel_view(panel, session),))
                    .style(style().size(pct(100), pct(100)).min_height(0))
            })
        },
    )
}
