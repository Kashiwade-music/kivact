//! Role: The Emulation panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The emulated theme; the protocol's `None` means no override.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ThemeChoice {
    /// No theme override.
    None,
    /// The light theme.
    Light,
    /// The dark theme.
    Dark,
}

impl ThemeChoice {
    /// The choice that shows `theme`.
    fn from_protocol(theme: Option<p::ThemeEmulation>) -> ThemeChoice {
        match theme {
            Some(p::ThemeEmulation::Light) => ThemeChoice::Light,
            Some(p::ThemeEmulation::Dark) => ThemeChoice::Dark,
            None => ThemeChoice::None,
        }
    }

    /// The emulated theme this choice means.
    fn to_protocol(self) -> Option<p::ThemeEmulation> {
        match self {
            ThemeChoice::None => None,
            ThemeChoice::Light => Some(p::ThemeEmulation::Light),
            ThemeChoice::Dark => Some(p::ThemeEmulation::Dark),
        }
    }

    /// The option label.
    fn label(self) -> &'static str {
        match self {
            ThemeChoice::None => "None",
            ThemeChoice::Light => "Light",
            ThemeChoice::Dark => "Dark",
        }
    }
}

/// The emulated pseudo-locale.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PseudoChoice {
    /// No pseudo-locale.
    Off,
    /// Accented letters.
    Accented,
    /// Accented letters with padding.
    Long,
    /// Right-to-left text.
    Rtl,
}

impl PseudoChoice {
    /// The choice that shows `pseudo_locale`.
    fn from_protocol(pseudo_locale: p::PseudoLocale) -> PseudoChoice {
        match pseudo_locale {
            p::PseudoLocale::Off => PseudoChoice::Off,
            p::PseudoLocale::Accented => PseudoChoice::Accented,
            p::PseudoLocale::Long => PseudoChoice::Long,
            p::PseudoLocale::Rtl => PseudoChoice::Rtl,
        }
    }

    /// The pseudo-locale this choice means.
    fn to_protocol(self) -> p::PseudoLocale {
        match self {
            PseudoChoice::Off => p::PseudoLocale::Off,
            PseudoChoice::Accented => p::PseudoLocale::Accented,
            PseudoChoice::Long => p::PseudoLocale::Long,
            PseudoChoice::Rtl => p::PseudoLocale::Rtl,
        }
    }

    /// The option label.
    fn label(self) -> &'static str {
        match self {
            PseudoChoice::Off => "Off",
            PseudoChoice::Accented => "Accented",
            PseudoChoice::Long => "Long",
            PseudoChoice::Rtl => "Right-to-left",
        }
    }
}

/// The environment emulation controls.
pub(crate) fn view(session: Session) -> impl IntoView {
    column((
        theme_select(session),
        text(
            "Theme emulation only affects apps that choose their theme from the platform theme hint.",
        ),
        motion_switch(session),
        show_some(
            move || session.emulation.latest(),
            move |emulation| {
                let locale = signal(String::new());
                let scale = signal(String::new());
                // The inputs follow the target's value, so a reset clears what they show.
                effect(move || {
                    let (next_locale, next_scale) = emulation.with(|emulation| {
                        (
                            emulation.locale.clone().unwrap_or_default(),
                            emulation
                                .scale
                                .map(|scale| scale.to_string())
                                .unwrap_or_default(),
                        )
                    });
                    locale.set(next_locale);
                    scale.set(next_scale);
                });
                column((locale_input(session, locale), scale_input(session, scale)))
                    .style(style().gap(12).align_items(Align::Start))
            },
        ),
        pseudo_select(session),
        touch_switch(session),
        button("Reset")
            .disabled(unavailable(session))
            .on_press(move || reset(session)),
    ))
    .style(style().gap(12).padding(12).align_items(Align::Start))
}

/// The theme select.
fn theme_select(session: Session) -> impl IntoView {
    select(
        "Theme",
        theme_options(),
        binding(
            move || {
                session
                    .emulation
                    .latest()
                    .map(|emulation| ThemeChoice::from_protocol(emulation.theme))
            },
            move |choice: Option<ThemeChoice>| {
                let Some(choice) = choice else {
                    return;
                };
                update(session, move |emulation| {
                    emulation.theme = choice.to_protocol();
                });
            },
        ),
    )
    .disabled(unavailable(session))
}

/// The reduced-motion switch.
fn motion_switch(session: Session) -> impl IntoView {
    switch(
        "Reduced motion",
        binding(
            move || {
                session
                    .emulation
                    .latest()
                    .is_some_and(|emulation| emulation.reduced_motion == Some(true))
            },
            move |on: bool| {
                update(session, move |emulation| {
                    emulation.reduced_motion = on.then_some(true);
                });
            },
        ),
    )
    .disabled(unavailable(session))
}

/// The locale input.
fn locale_input(session: Session, locale: Signal<String>) -> impl IntoView {
    text_input("Locale", locale)
        .on_draft_commit(move |text| {
            let locale = text.trim();
            let locale = (!locale.is_empty()).then(|| locale.to_owned());
            update(session, move |emulation| emulation.locale = locale);
        })
        .disabled(unavailable(session))
}

/// The pseudo-locale select.
fn pseudo_select(session: Session) -> impl IntoView {
    select(
        "Pseudo-locale",
        pseudo_options(),
        binding(
            move || {
                session
                    .emulation
                    .latest()
                    .map(|emulation| PseudoChoice::from_protocol(emulation.pseudo_locale))
            },
            move |choice: Option<PseudoChoice>| {
                let Some(choice) = choice else {
                    return;
                };
                update(session, move |emulation| {
                    emulation.pseudo_locale = choice.to_protocol();
                });
            },
        ),
    )
    .disabled(unavailable(session))
}

/// The DPI-scale input; an empty text removes the emulated scale.
fn scale_input(session: Session, scale: Signal<String>) -> impl IntoView {
    text_input("DPI scale", scale)
        .on_draft_commit(move |draft| {
            let draft = draft.trim();
            if draft.is_empty() {
                update(session, |emulation| emulation.scale = None);
                return;
            }
            match draft.parse::<f64>() {
                Ok(scale) if scale.is_finite() && scale > 0.0 => {
                    update(session, move |emulation| emulation.scale = Some(scale));
                }
                _ => session.report("the DPI scale must be a positive number".to_owned()),
            }
        })
        .disabled(unavailable(session))
}

/// The touch switch.
fn touch_switch(session: Session) -> impl IntoView {
    switch(
        "Touch",
        binding(
            move || {
                session
                    .emulation
                    .latest()
                    .is_some_and(|emulation| emulation.touch == Some(true))
            },
            move |on: bool| {
                update(session, move |emulation| {
                    emulation.touch = on.then_some(true)
                });
            },
        ),
    )
    .disabled(unavailable(session))
}

/// Whether the emulation is not loaded yet.
fn unavailable(session: Session) -> impl Fn() -> bool + Copy {
    move || session.emulation.latest().is_none()
}

/// Send the whole emulation with one change, then read it back.
fn update(session: Session, edit: impl FnOnce(&mut p::Emulation)) {
    let Some(mut emulation) = session.emulation.latest() else {
        return;
    };
    edit(&mut emulation);
    session.send::<p::SetEmulation>(emulation, move |_| session.emulation.refetch());
}

/// Send an emulation with no setting: the target shows its real environment again.
fn reset(session: Session) {
    session.send::<p::SetEmulation>(
        p::Emulation {
            theme: None,
            reduced_motion: None,
            locale: None,
            pseudo_locale: p::PseudoLocale::Off,
            scale: None,
            touch: None,
        },
        move |_| session.emulation.refetch(),
    );
}

/// The options of the theme select.
fn theme_options() -> Vec<ListItem<ThemeChoice>> {
    [ThemeChoice::None, ThemeChoice::Light, ThemeChoice::Dark]
        .into_iter()
        .map(|choice| ListItem::new(choice, choice.label()))
        .collect()
}

/// The options of the pseudo-locale select.
fn pseudo_options() -> Vec<ListItem<PseudoChoice>> {
    [
        PseudoChoice::Off,
        PseudoChoice::Accented,
        PseudoChoice::Long,
        PseudoChoice::Rtl,
    ]
    .into_iter()
    .map(|choice| ListItem::new(choice, choice.label()))
    .collect()
}
