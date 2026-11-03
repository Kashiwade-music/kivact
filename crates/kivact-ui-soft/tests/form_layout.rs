//! Role: Check shared label tracks and the narrow form breakpoint.
use kivact::control::{NodeBuilder, text};
use kivact::signal;
use kivact::style::{ThemeMode, style};
use kivact::test::{RenderOptions, WindowMode, render_with};
use kivact::ui::text_input;
use kivact::view::Role;
use kivact_ui_soft::{FormDirection, form_layout, form_row, theme};

#[test]
fn horizontal_labels_share_a_track_and_stack_at_480_pixels() {
    let screen = render_with(
        RenderOptions {
            width: 620.0,
            height: 400.0,
            theme: theme(ThemeMode::Light),
            window: WindowMode::Headless,
            ..RenderOptions::from_env()
        },
        || {
            form_layout((
                form_row(text("Name"), text_input("Name", signal(String::new()))),
                form_row(
                    text("Delivery address"),
                    text_input("Delivery address", signal(String::new())),
                ),
            ))
            .direction(FormDirection::HorizontalLabels)
            .style(style().width(400))
        },
    );
    let name = screen.get_by_role(Role::TextBox, "Name");
    let address = screen.get_by_role(Role::TextBox, "Delivery address");
    let label = screen.get_by_role(Role::Text, "Delivery address");
    let first = name.rect().unwrap();
    let second = address.rect().unwrap();
    let long_label = label.rect().unwrap();
    assert!((first.x - second.x).abs() < 1.0);
    assert!((second.x - long_label.x - long_label.width - 16.0).abs() < 1.0);
    assert!((second.y - first.y - first.height - 12.0).abs() < 1.0);

    screen.resize(480.0, 400.0);
    let second = address.rect().unwrap();
    let long_label = label.rect().unwrap();
    assert!((second.x - long_label.x).abs() < 1.0);
    assert!((second.y - long_label.y - long_label.height - 16.0).abs() < 1.0);
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
