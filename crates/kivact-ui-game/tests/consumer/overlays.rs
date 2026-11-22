//! Role: Modal menus, command search, toasts, tooltips and dialogs via the public API.

use kivact::interaction::Tone;
use kivact::prelude::*;
use kivact::view::{rich_text, span};
use kivact_ui_game as g;
use std::time::Duration;

#[test]
fn consumer_toast_expiry_and_hover_pause() {
    let (screen, toaster) = kivact::test::render_with_handle(
        kivact::test::RenderOptions::with_theme(kivact_ui_game::theme(
            kivact::style::ThemeMode::Light,
        ))
        .reduced_motion(),
        || {
            let toaster = kivact::interaction::use_toaster().max_visible(4);
            (
                toaster,
                (
                    column((
                        kivact::ui::button("Notify")
                            .variant(kivact::ui::Slant)
                            .on_press(move || {
                                toaster.push(
                                    Toast::new("Saved")
                                        .body("Ready")
                                        .tone(Tone::Success)
                                        .duration(Some(Duration::from_millis(400))),
                                );
                            }),
                        service_notice(),
                        g::activity(vec![g::ActivityEntry::new(
                            "09:40",
                            rich_text(vec![span("Review").font_weight(700), span(" passed")]),
                            kivact::interaction::Tone::Success,
                        )]),
                    )),
                    kivact::ui::toast_region(toaster),
                ),
            )
        },
    );
    assert!(screen.get_by_text("Maintenance: read only").exists());
    assert!(screen.get_by_text("Review passed").exists());
    screen.get_by_role(Role::Button, "Notify").click();
    assert!(screen.get_by_role(Role::Status, "Saved").exists());
    screen.get_by_text("Ready").hover();
    screen.advance(Duration::from_secs(1));
    assert_eq!(toaster.toasts().get_untracked().len(), 1);
    screen.mouse_leave();
    screen.advance(Duration::from_millis(399));
    assert_eq!(toaster.toasts().get_untracked().len(), 1);
    screen.advance(Duration::from_millis(1));
    assert!(toaster.toasts().get_untracked().is_empty());
    for _ in 0..5 {
        screen.get_by_role(Role::Button, "Notify").click();
    }
    assert_eq!(toaster.toasts().get_untracked().len(), 4, "four are shown");
    screen.advance(Duration::from_millis(400));
    assert_eq!(
        toaster.toasts().get_untracked().len(),
        1,
        "the fifth takes its place"
    );
    screen.advance(Duration::from_millis(400));
    assert!(toaster.toasts().get_untracked().is_empty());
    toaster.push(Toast::new("Failed").body("Rejected").tone(Tone::Danger));
    screen.run_until_idle();
    screen.get_by_role(Role::Alert, "Failed");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn service_notice() -> impl IntoView {
    kivact::ui::alert("NOTICE")
        .variant(kivact::ui::AlertVariant::Banner)
        .tone(kivact::interaction::Tone::Warning)
        .icon(kivact::view::icon("alert-triangle"))
        .actions(())
        .icon(
            g::mono("NOTICE").style(
                style()
                    .font_size(10)
                    .font_weight(800)
                    .padding_x(6)
                    .padding_y(4)
                    .background(g::color::YELLOW),
            ),
        )
        .with_node(|node| {
            node.role(Role::Status)
                .live_region(kivact::view::LiveRegion::Polite)
        })
        .content(move || {
            rich_text(vec![
                span("Maintenance:").font_weight(700).color(rgb(0xffc928)),
                span(" read only"),
            ])
        })
        .name("Service notice")
}
