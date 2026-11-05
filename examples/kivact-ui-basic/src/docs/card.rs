//! Role: Catalog page of Card: login form, small, bordered header and image.
use kivact::ui::ButtonVariant;

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact_ui_basic as basic;

pub(crate) fn page() -> Page {
    Page {
        slug: "card",
        name: "Card",
        description: "見出し、内容、フッターを持つ枠付きの表示領域です。header、image、content、footer の各スロットに渡します。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::ButtonVariant;\nuse kivact_ui_basic as basic;",
        source: include_str!("card.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "card_header に見出しと説明、右上の操作を渡し、content にフォーム、footer にボタンを置いたログインの例です。",
                default,
            ),
            Example::new(
                "small",
                "Small",
                "size に Sm を渡すと、余白が 12 px になります。",
                small,
            ),
            Example::new(
                "bordered",
                "Bordered header",
                "bordered で見出しの下に線を引きます。",
                bordered,
            ),
            Example::new(
                "image",
                "Image",
                "image は上端いっぱいに表示され、上の余白がなくなります。",
                with_image,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    let email = signal(String::new());
    let password = signal(String::new());
    kivact::ui::card({
        basic::field_group((
            kivact::ui::field(
                "Email",
                text_input("Email", email).placeholder("m@example.com"),
            ),
            kivact::ui::field(
                "Password",
                text_input("Password", password).purpose(kivact::view::InputPurpose::Password),
            ),
        ))
    })
    .header(
        row((
            column((
                text("Login to your account")
                    .style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
                text("Enter your email below to login to your account").style(
                    style()
                        .line_height(1.625)
                        .color(basic::ColorName::MutedForeground),
                ),
            ))
            .style(style().gap(4).grow(1).min_width(0)),
            row(button("Sign Up").variant(ButtonVariant::Link))
                .style(style().align_self(Align::Start).shrink(0)),
        ))
        .style(style().align_items(Align::Start).gap(8)),
    )
    .footer({
        column((
            button("Login").style(style().width(pct(100))),
            button("Login with Google")
                .variant(ButtonVariant::Outline)
                .style(style().width(pct(100))),
        ))
        .style(style().gap(8).width(pct(100)))
    })
    .style(style().width(360))
}
// end example

// example: small
fn small() -> impl IntoView {
    kivact::ui::card(text("Content"))
        .variant(basic::CardSize::Sm)
        .header(
            column((
                text("Small card")
                    .style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
                text("Tighter spacing.").style(
                    style()
                        .line_height(1.625)
                        .color(basic::ColorName::MutedForeground),
                ),
            ))
            .style(style().gap(4).grow(1).min_width(0)),
        )
        .style(style().width(360))
}
// end example

// example: bordered
fn bordered() -> impl IntoView {
    kivact::ui::card(text("You have 3 unread messages."))
        .variant(basic::CardLook::default().header_bordered(true))
        .header(
            column((text("Notifications")
                .style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),))
            .style(style().gap(4).grow(1).min_width(0)),
        )
        .style(style().width(360))
}
// end example

// example: image
fn with_image() -> impl IntoView {
    let landscape: &'static [u8] = include_bytes!("../../assets/images/landscape.png");
    kivact::ui::card(())
        .media(image(landscape).style(style().width(pct(100)).aspect_ratio(16.0 / 9.0)))
        .header(
            column((
                text("Design systems meetup")
                    .style(style().font_size(14).line_height(20.0 / 14.0).weight(500)),
                text("A practical talk.").style(
                    style()
                        .line_height(1.625)
                        .color(basic::ColorName::MutedForeground),
                ),
            ))
            .style(style().gap(4).grow(1).min_width(0)),
        )
        .footer(button("View Event").style(style().width(pct(100))))
        .style(style().width(360))
}
// end example
