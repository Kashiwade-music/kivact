//! Role: Real-window checks for common kit constructor and themed field contracts.
use super::*;
fn page(child: impl IntoView) -> Node {
    column(child).style(style().padding(16).gap(12))
}

#[test]
fn pagination_clamps_display_and_actions_without_normalizing_external_binding() {
    let (screen, (index, count)) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let index = signal(999usize);
            let count = signal(5usize);
            ((index, count), page(basic::pagination(index, count)))
        },
    );
    assert_eq!(index.get_untracked(), 999);
    assert!(screen.get_by_role(Role::Link, "5").is_selected());
    assert!(
        screen
            .get_by_role(Role::Link, "Go to next page")
            .is_disabled()
    );
    screen
        .get_by_role(Role::Link, "Go to previous page")
        .click();
    assert_eq!(index.get_untracked(), 3);
    count.set(2);
    screen.run_until_idle();
    assert_eq!(index.get_untracked(), 3);
    assert!(screen.get_by_role(Role::Link, "2").is_selected());
    screen
        .get_by_role(Role::Link, "Go to previous page")
        .click();
    assert_eq!(index.get_untracked(), 0);
    count.set(0);
    screen.run_until_idle();
    assert!(
        screen
            .get_by_role(Role::Link, "Go to previous page")
            .is_disabled()
    );
    assert!(
        screen
            .get_by_role(Role::Link, "Go to next page")
            .is_disabled()
    );
    assert!(screen.query_by_role(Role::Link, "1").is_none());
    assert_clean(&screen);
}

#[test]
fn standard_field_keeps_visible_name_hint_and_error_under_kit_theme() {
    let (screen, error) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let error = signal(None::<String>);
            (
                error,
                kivact::ui::field(
                    "Visible name",
                    kivact::ui::text_input("Old name", signal(String::new())),
                )
                .hint("Helpful text")
                .error(error),
            )
        },
    );
    let input = || screen.get_by_role(Role::TextBox, "Visible name");
    assert!(!input().invalid());
    assert_eq!(
        input()
            .described_by()
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>(),
        ["Helpful text"]
    );
    error.set(Some("Required".to_owned()));
    screen.run_until_idle();
    assert!(input().invalid());
    assert_eq!(
        input()
            .described_by()
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>(),
        ["Helpful text", "Required"]
    );
    screen.get_by_role(Role::Alert, "Required");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn standard_field_label_activates_controls_and_keeps_disabled_or_hidden_targets_inert() {
    let (screen, (enabled, blocked, hidden, disabled, caller_focused)) =
        kivact::test::render_with_handle(
            RenderOptions {
                theme: basic::theme(kivact::style::ThemeMode::Light),
                reduced_motion: true,
                ..RenderOptions::from_env()
            },
            || {
                let enabled = signal(false);
                let blocked = signal(false);
                let hidden = signal(false);
                let disabled = signal(true);
                let caller_ref = node_ref();
                let caller_focused = caller_ref.focused();
                (
                    (enabled, blocked, hidden, disabled, caller_focused),
                    page((
                        kivact::ui::field(
                            "Toggle label",
                            kivact::ui::switch("Old toggle", enabled).hide_label(true),
                        ),
                        kivact::ui::field(
                            "Input label",
                            kivact::ui::text_input("Old input", signal(String::new()))
                                .node_ref(caller_ref),
                        ),
                        kivact::ui::field(
                            "Disabled label",
                            kivact::ui::checkbox("Old disabled", blocked)
                                .disabled(disabled)
                                .hide_label(true),
                        ),
                        kivact::ui::field(
                            "Hidden label",
                            kivact::ui::checkbox("Old hidden", hidden)
                                .hidden(true)
                                .hide_label(true),
                        ),
                    )),
                )
            },
        );
    screen.get_by_text("Toggle label").click();
    assert!(
        enabled.get_untracked(),
        "the visible label activates its switch"
    );
    assert!(
        screen
            .get_by_role(Role::Switch, "Toggle label")
            .is_focused()
    );
    screen.get_by_text("Input label").click();
    assert!(
        screen
            .get_by_role(Role::TextBox, "Input label")
            .is_focused()
    );
    assert!(
        caller_focused.get_untracked(),
        "the caller's ref remains bound"
    );
    screen.get_by_text("Disabled label").click();
    screen.get_by_text("Hidden label").click();
    assert!(!blocked.get_untracked());
    assert!(!hidden.get_untracked());
    disabled.set(false);
    screen.run_until_idle();
    screen.get_by_text("Disabled label").click();
    assert!(blocked.get_untracked());
    assert_clean(&screen);
}

#[test]
fn standard_field_number_label_focuses_the_value_without_stepping() {
    let (screen, value) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let value = signal(3_f64);
            (
                value,
                page(kivact::ui::field(
                    "Amount label",
                    kivact::ui::number_input(
                        "Old amount",
                        value,
                        kivact::ui::ScalarRange::new(0., 10.).unwrap(),
                    ),
                )),
            )
        },
    );
    screen.get_by_text("Amount label").click();
    assert_eq!(
        value.get_untracked(),
        3.,
        "a field label must not step a composite input"
    );
    assert!(
        screen
            .get_by_role(Role::SpinButton, "Amount label")
            .is_focused()
    );
    assert_clean(&screen);
}

#[test]
fn field_errors_reach_number_input_spinbutton_and_grid_keeps_visual_step_order() {
    let (screen, error) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let error = signal(None::<String>);
            (
                error,
                page(
                    kivact::ui::field(
                        "Number",
                        kivact::ui::number_input(
                            "Value",
                            signal(3_f64),
                            kivact::ui::ScalarRange::new(0., 10.).unwrap(),
                        ),
                    )
                    .hint("Number hint")
                    .error(error),
                ),
            )
        },
    );
    let input = screen.get_by_role(Role::SpinButton, "Number");
    assert!(!input.invalid());
    error.set(Some("Invalid number".to_owned()));
    screen.run_until_idle();
    assert!(input.invalid());
    assert_eq!(
        (
            input
                .labelled_by()
                .iter()
                .map(|e| e.text())
                .collect::<Vec<_>>(),
            input
                .described_by()
                .iter()
                .map(|e| e.text())
                .collect::<Vec<_>>()
        ),
        (
            vec!["Number".to_owned()],
            vec!["Number hint".to_owned(), "Invalid number".to_owned()]
        )
    );
    let left = screen.get_by_role(Role::Button, "Decrease").rect().unwrap();
    let value = input.rect().unwrap();
    let right = screen.get_by_role(Role::Button, "Increase").rect().unwrap();
    assert!(left.x < value.x && value.x < right.x);
    assert_clean(&screen);
}

#[test]
fn standard_field_tag_label_focuses_editor_without_removing_a_tag() {
    let (screen, tags) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let tags = signal(vec!["alpha".to_owned(), "beta".to_owned()]);
            (
                tags,
                page(kivact::ui::field(
                    "Tag label",
                    kivact::ui::tag_input(
                        "Old tags",
                        tags,
                        move |tag| tags.update(|items| items.push(tag)),
                        move |index| {
                            tags.update(|items| {
                                items.remove(index);
                            })
                        },
                    ),
                )),
            )
        },
    );
    screen.get_by_text("Tag label").click();
    assert_eq!(
        tags.get_untracked(),
        ["alpha", "beta"],
        "label activation must not remove tags"
    );
    assert!(screen.get_by_role(Role::TextBox, "Tag label").is_focused());
    assert_clean(&screen);
}

#[test]
fn tag_input_keeps_wrapped_chips_inside_narrow_bounds_and_forwards_validation() {
    let (screen, error) = kivact::test::render_with_handle(
        RenderOptions {
            theme: basic::theme(kivact::style::ThemeMode::Light),
            reduced_motion: true,
            ..RenderOptions::from_env()
        },
        || {
            let error = signal(None::<String>);
            let tags = signal(vec![
                "alpha".to_owned(),
                "beta".to_owned(),
                "gamma".to_owned(),
                "delta".to_owned(),
            ]);
            (
                error,
                page(
                    kivact::ui::field(
                        "Narrow tags",
                        kivact::ui::tag_input(
                            "Tags",
                            tags,
                            move |tag| tags.update(|items| items.push(tag)),
                            move |index| {
                                tags.update(|items| {
                                    items.remove(index);
                                })
                            },
                        )
                        .test_id("narrow-tags")
                        .style(style().width(260)),
                    )
                    .hint("Tag hint")
                    .error(error),
                ),
            )
        },
    );
    let input = screen.get_by_role(Role::TextBox, "Narrow tags");
    let outer = screen.get_by_test_id("narrow-tags").rect().unwrap();
    let alpha = screen.get_by_text("alpha").rect().unwrap();
    let delta = screen.get_by_text("delta").rect().unwrap();
    assert!(delta.y > alpha.y, "chips wrap in their first grid column");
    for element in [
        screen.get_by_text("alpha"),
        screen.get_by_text("beta"),
        screen.get_by_text("gamma"),
        screen.get_by_text("delta"),
        input,
    ] {
        let rect = element.rect().unwrap();
        assert!(rect.x >= outer.x && rect.x + rect.width <= outer.x + outer.width + 1.);
    }
    error.set(Some("Invalid tags".to_owned()));
    screen.run_until_idle();
    assert!(input.invalid());
    assert_eq!(
        (
            input
                .labelled_by()
                .iter()
                .map(|e| e.text())
                .collect::<Vec<_>>(),
            input
                .described_by()
                .iter()
                .map(|e| e.text())
                .collect::<Vec<_>>()
        ),
        (
            vec!["Narrow tags".to_owned()],
            vec!["Tag hint".to_owned(), "Invalid tags".to_owned()]
        )
    );
    screen.assert_layout();
    assert_clean(&screen);
}
