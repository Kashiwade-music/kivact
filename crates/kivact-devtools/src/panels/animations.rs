//! Role: The Animations panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The running animations and the animation clock.
pub(crate) fn view(session: Session) -> impl IntoView {
    let data = signal(None::<p::AnimationList>);
    let scale = signal(Some("1".to_owned()));
    on_mount(move || refresh(session, data, scale));

    let rows = memo(move || data.get().map(|list| list.animations).unwrap_or_default());
    column((
        row((
            text("Animations"),
            toggle_group("Time scale", scale_items(), scale).on_edit(move |phase| {
                if phase != EditPhase::Commit {
                    return;
                }
                let Some(key) = scale.get_untracked() else {
                    return;
                };
                let scale_value = key.parse::<f64>().unwrap_or(1.0);
                session.send::<p::SetTimeScale>(
                    p::TimeScaleParams { scale: scale_value },
                    move |_| refresh(session, data, scale),
                );
            }),
        )),
        data_table(
            rows,
            |animation: &p::AnimationInfo| animation.id,
            vec![
                DataColumn::new("Id", |animation: &p::AnimationInfo| {
                    animation.id.to_string()
                }),
                DataColumn::new("Kind", |animation: &p::AnimationInfo| {
                    kind_text(animation.kind)
                }),
                DataColumn::new("Owner", |animation: &p::AnimationInfo| {
                    animation.owner.to_string()
                }),
                DataColumn::new("Running", |animation: &p::AnimationInfo| {
                    yes_no(animation.running)
                }),
                DataColumn::new("Paused", |animation: &p::AnimationInfo| {
                    yes_no(animation.paused)
                }),
                DataColumn::new("Decorative", |animation: &p::AnimationInfo| {
                    yes_no(animation.decorative)
                }),
            ],
        )
        .row_label(|animation| format!("#{}", animation.id))
        .name("Running animations")
        .style(style().grow(1.0).min_height(0.0)),
    ))
    .style(style().grow(1.0).min_height(0.0))
}

/// Read the animations and set the selected time scale from the answer.
fn refresh(
    session: Session,
    data: Signal<Option<p::AnimationList>>,
    scale: Signal<Option<String>>,
) {
    session.send::<p::ListAnimations>(p::Empty {}, move |list| {
        scale.set(Some(scale_key(list.time_scale)));
        data.set(Some(list));
    });
}

/// The choices of the time scale toggle group.
fn scale_items() -> Vec<ListItem<String>> {
    [
        ("0", "Stop"),
        ("0.1", "0.1x"),
        ("0.25", "0.25x"),
        ("0.5", "0.5x"),
        ("1", "1x"),
    ]
    .into_iter()
    .map(|(key, label)| ListItem::new(key.to_owned(), label))
    .collect()
}

/// The key of the toggle group for a time scale.
fn scale_key(scale: f64) -> String {
    if scale <= 0.0 {
        "0".to_owned()
    } else {
        format!("{scale}")
    }
}

/// The wire name of an animation kind.
fn kind_text(kind: p::AnimationKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "animated".to_owned())
}

/// `yes` or `no`.
fn yes_no(value: bool) -> String {
    if value {
        "yes".to_owned()
    } else {
        "no".to_owned()
    }
}
