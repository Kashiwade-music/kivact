//! Role: Apply Pop bubble, log, composer, typing and source-code appearances.
use crate::Tokens;
use kivact::geometry::Rect;
use kivact::style::{Align, Color, Direction, Display, Length, Theme, Track, pct, style};
use kivact::ui::{
    ChatAlign, ChatBubbleGroup, ChatComposerRecipes, ChatMessageRecipes, ChatMessageState,
    ChatMessageVariant, ChatThreadRecipes, CodeBlockRecipes, Look, TypingIndicatorRecipes,
    TypingIndicatorState, Variant,
};

/// Compact business-conversation bubbles with a hidden visible author line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopBubble;
impl Variant for PopBubble {}

pub(crate) fn message(r: ChatMessageRecipes) -> ChatMessageRecipes {
    let r = r
        .root(|prev| {
            move |t: &Theme, s: &Look<ChatMessageState>| {
                let base = prev(t, s).gap(10).align_items(Align::Start).min_width(0);
                if s.variant::<PopBubble>().is_some() {
                    base.max_width(pct(78))
                        .align_self(if s.align == ChatAlign::End {
                            Align::End
                        } else {
                            Align::Start
                        })
                        .padding_left(
                            if s.align == ChatAlign::Start
                                && matches!(
                                    s.group,
                                    ChatBubbleGroup::Middle | ChatBubbleGroup::Last
                                )
                            {
                                34
                            } else {
                                0
                            },
                        )
                } else {
                    base
                }
            }
        })
        .body(|prev| {
            move |t: &Theme, s: &Look<ChatMessageState>| {
                let base = prev(t, s).radius(14).gap(4);
                if s.variant::<PopBubble>().is_some() {
                    base.padding(0)
                        .background(Color::TRANSPARENT)
                        .max_width(pct(100))
                } else if s.variant::<ChatMessageVariant>() == Some(&ChatMessageVariant::Plain) {
                    base
                } else {
                    let p = t.ext::<Tokens>();
                    base.background(if s.align == ChatAlign::End {
                        p.blue
                    } else {
                        p.surface_active
                    })
                    .color(if s.align == ChatAlign::End {
                        Color::WHITE
                    } else {
                        p.text
                    })
                }
            }
        });
    message_details(r)
}

fn message_details(r: ChatMessageRecipes) -> ChatMessageRecipes {
    r.author(|prev| {
        move |t: &Theme, s: &Look<ChatMessageState>| {
            let base = prev(t, s);
            if s.variant::<PopBubble>().is_some() {
                base.position(kivact::style::Position::Absolute)
                    .visibility(kivact::style::Visibility::Hidden)
            } else {
                base.color(t.ext::<Tokens>().text)
            }
        }
    })
    .avatar(|prev| {
        move |t: &Theme, s: &Look<ChatMessageState>| {
            let base = prev(t, s);
            if s.variant::<PopBubble>().is_some() && s.align == ChatAlign::End {
                base.position(kivact::style::Position::Absolute)
                    .visibility(kivact::style::Visibility::Hidden)
            } else {
                base
            }
        }
    })
    .content(|prev| {
        move |t: &Theme, s: &Look<ChatMessageState>| {
            let base = prev(t, s);
            if s.variant::<PopBubble>().is_none() {
                return base;
            }
            let p = t.ext::<Tokens>();
            let mine = s.align == ChatAlign::End;
            let first = matches!(s.group, ChatBubbleGroup::Single | ChatBubbleGroup::First);
            base.font_size(14)
                .padding_x(12)
                .padding_y(8)
                .radius_corners(
                    if !mine && first { 4 } else { 14 },
                    if mine && first { 4 } else { 14 },
                    14,
                    14,
                )
                .background(if mine { p.blue } else { p.surface_active })
                .color(if mine { Color::WHITE } else { p.text })
        }
    })
    .time(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .font_size(11)
                .padding_x(4)
                .align_self(Align::End)
                .color(t.ext::<Tokens>().text_muted)
        }
    })
}
pub(crate) fn thread(r: ChatThreadRecipes) -> ChatThreadRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).background(t.ext::<Tokens>().surface))
        .scroll_button(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .radius(8)
                    .background(t.ext::<Tokens>().surface)
                    .border(1, t.ext::<Tokens>().line)
            }
        })
}
pub(crate) fn composer(r: ChatComposerRecipes) -> ChatComposerRecipes {
    r.root(|prev| move |t: &Theme, s: &_| prev(t, s).border_top(1, t.ext::<Tokens>().line))
        .surface(|_| {
            |t: &Theme, _: &_| {
                style()
                    .display(Display::Grid)
                    .grid_template_columns((Length::Auto, Track::fr(1), Length::Auto))
                    .gap(8)
                    .padding(12)
                    .background(t.ext::<Tokens>().surface)
            }
        })
        .header(|_| {
            |_: &Theme, _: &_| {
                style()
                    .grid_column(1)
                    .grid_row(1)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
            }
        })
        .input(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .grid_column(2)
                    .grid_row(1)
                    .height(36)
                    .min_height(0)
                    .grow(1)
            }
        })
        .footer(|_| {
            |_: &Theme, _: &_| {
                style()
                    .grid_column(3)
                    .grid_row(1)
                    .direction(Direction::Row)
                    .gap(8)
                    .align_items(Align::Center)
            }
        })
        .send(|prev| {
            move |t: &Theme, s: &_| {
                prev(t, s)
                    .background(t.ext::<Tokens>().pink)
                    .color(Color::WHITE)
                    .radius(8)
            }
        })
}
pub(crate) fn typing(r: TypingIndicatorRecipes) -> TypingIndicatorRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .gap(10)
                .align_items(Align::Start)
                .max_width(pct(78))
        }
    })
    .dots(|_| {
        |t: &Theme, s: &Look<TypingIndicatorState>| {
            let (phase, color) = (s.phase, t.ext::<Tokens>().text_faint);
            style()
                .size(54, 30)
                .radius_corners(4, 14, 14, 14)
                .background(t.ext::<Tokens>().surface_active)
                .paint(move |d, _| {
                    for index in 0..3 {
                        let bounce = (phase - index as f64 * 0.95).sin().max(0.0) * 3.0;
                        d.rounded_rect(
                            Rect::new(14.0 + index as f64 * 10.0, 12.0 - bounce, 6.0, 6.0),
                            3.0,
                            color,
                        );
                    }
                })
        }
    })
}
pub(crate) fn code(r: CodeBlockRecipes) -> CodeBlockRecipes {
    r.root(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .radius(10)
                .border_color(t.ext::<Tokens>().line)
                .background(t.ext::<Tokens>().surface)
        }
    })
    .header(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .background(t.ext::<Tokens>().surface_muted)
                .padding_x(12)
                .padding_y(8)
        }
    })
    .code(|prev| move |t: &Theme, s: &_| prev(t, s).font_size(13).color(t.ext::<Tokens>().text))
    .gutter(|prev| {
        move |t: &Theme, s: &_| {
            prev(t, s)
                .color(t.ext::<Tokens>().text_faint)
                .background(t.ext::<Tokens>().surface_muted)
        }
    })
}
