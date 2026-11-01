//! Role: Consumer checks for application-owned content and numeric presentation.
use super::*;
use kivact::ui::toaster;
use kivact::view::node_ref;

#[test]
fn kpi_uses_plain_decimal_text_and_application_trend_and_formatter() {
    let (screen, (trend, root, default_root)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let trend = signal(Vec::<f64>::new());
            let root = node_ref();
            let default_root = node_ref();
            (
                (trend, root, default_root),
                column((
                    g::kpi("Count", 1284.25)
                        .decimals(2)
                        .unit("")
                        .change("")
                        .tone(kivact::ui::Tone::Info)
                        .trend(trend)
                        .node_ref(root)
                        .style(style().width(320)),
                    g::kpi("Count", 1284.25)
                        .decimals(2)
                        .unit("")
                        .change("")
                        .tone(kivact::ui::Tone::Info)
                        .node_ref(default_root)
                        .style(style().width(320)),
                    g::kpi("Custom", 54.)
                        .decimals(0)
                        .unit("")
                        .change("")
                        .tone(kivact::ui::Tone::Info)
                        .heading(|| text("Custom"))
                        .format(|value| format!("#{value:.1}"))
                        .style(style().width(320)),
                )),
            )
        });
    assert_eq!(screen.all_by_text("1284.25").len(), 2);
    screen.get_by_text("#54.0");
    assert!(screen.query_by_text("1,284.25").is_none());
    let empty_height = root.rect().get_untracked().unwrap().height;
    // Siblings at different origins can snap to adjacent pixel heights.
    assert!((default_root.rect().get_untracked().unwrap().height - empty_height).abs() <= 1.);
    assert_eq!(root.rect().get_untracked().unwrap().width, 320.);
    trend.set(vec![2., 4., 1., 5.]);
    screen.run_until_idle();
    assert!(root.rect().get_untracked().unwrap().height > empty_height);
    let first = screen.screenshot().hash();
    trend.set(vec![5., 1., 4., 2.]);
    assert_ne!(first, screen.screenshot().hash());
    trend.set(Vec::new());
    screen.run_until_idle();
    assert!((root.rect().get_untracked().unwrap().height - empty_height).abs() <= 1.);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn dropzone_hides_empty_guidance_and_keeps_numeric_accessibility_precision() {
    let (screen, (hint, drop_root)) =
        kivact::test::render_with_handle(crate::options(true), || {
            let hint = signal(String::new());
            let drop_root = node_ref();
            (
                (hint, drop_root),
                column((
                    g::dropzone(kivact::i18n::message("game.dropzone.label", "Upload files"))
                        .on_files(|_| ())
                        .node_ref(drop_root)
                        .hint(hint)
                        .filters(vec![("Text".to_owned(), vec!["txt".to_owned()])])
                        .style(style().width(320)),
                    kivact::ui::slider("Exact slider", signal(12.345)),
                    kivact::ui::progress(Some(0.12345)).name("Exact progress"),
                    g::ring("Ring", 12.345, kivact::ui::Tone::Info),
                ))
                .style(style().width(360)),
            )
        });
    assert!(screen.query_by_text("CSV / JSON").is_none());
    assert_eq!(drop_root.rect().get_untracked().unwrap().width, 320.);
    let empty_height = drop_root.rect().get_untracked().unwrap().height;
    hint.set("Text documents".to_owned());
    screen.run_until_idle();
    screen.get_by_text("Text documents");
    assert!(drop_root.rect().get_untracked().unwrap().height > empty_height);
    hint.set(String::new());
    screen.run_until_idle();
    assert!((drop_root.rect().get_untracked().unwrap().height - empty_height).abs() <= 1.);
    assert!(screen.query_by_text("Text documents").is_none());
    assert_eq!(
        screen
            .get_by_role(Role::Slider, "Exact slider")
            .value()
            .as_deref(),
        Some("12.345")
    );
    let progress = screen.get_by_role(Role::ProgressBar, "Exact progress");
    assert_eq!(progress.value_range().2, Some(0.12345));
    assert_eq!(progress.value().as_deref(), Some("12%"));
    assert_eq!(
        screen.get_by_role(Role::Meter, "").value().as_deref(),
        Some("12%")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn toast_region_preserves_the_app_limit_and_trailing_badge_tone_is_explicit() {
    let (screen, visible) = kivact::test::render_with_handle(crate::options(true), || {
        let toaster = toaster().max_visible(2);
        let visible = toaster.visible(false);
        let region = kivact::ui::toast_region(toaster);
        for index in 0..5 {
            toaster.push(Toast::new(format!("Notice {index}")).duration(None));
        }
        (visible, region)
    });
    assert_eq!(visible.get_untracked().len(), 2);
    screen.get_by_text("Notice 0");
    screen.get_by_text("Notice 1");
    assert!(screen.query_by_text("Notice 2").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    drop(screen);
    let render = |tone| {
        render_with(options(true), move || {
            kivact::ui::tabs(
                "Tabs",
                vec![ListItem::new("alert", "Alerts")],
                signal("alert"),
            )
            .orientation(kivact::view::Orientation::Vertical)
            .variant(g::Navigation)
            .item(move |item, state| (g::navigation_label(item, state), g::badge("!", tone)))
            .style(style().width(220))
        })
    };
    let info = render(kivact::ui::Tone::Info);
    let first = info.screenshot().hash();
    drop(info);
    let danger = render(kivact::ui::Tone::Danger);
    assert_ne!(first, danger.screenshot().hash());
    assert!(danger.errors().is_empty(), "{:?}", danger.errors());
}
