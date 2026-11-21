//! Role: Install Astryx Neutral's recipes of the chat group: the message, the thread and the composer.
use crate::color as c;
use crate::radius as r;
use crate::recipe_parts::shadowed;
use crate::shadows;
use crate::variants::{ChatBubbleVariant, ChatDensity, ComposerElevation, SystemMessageVariant};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Style, Theme, ThemeMode, color, pct, style,
};
use kivact::ui::theme::{
    ChatComposerState, ChatComposerTheme, ChatMessageState, ChatMessageTheme, ChatThreadState,
    ChatThreadTheme, Look,
};
use kivact::ui::{ChatBubbleGroup, ChatMessageVariant, ChatSender, ComposerStatus};

/// The density of a chat part: its variant, else balanced.
fn density<S>(s: &Look<S>) -> ChatDensity {
    s.variant::<ChatDensity>().unwrap_or_default()
}

/// The gap between the parts of a message or a thread at `density`, in logical px.
fn gap(d: ChatDensity) -> i32 {
    match d {
        ChatDensity::Compact => 8,
        ChatDensity::Balanced => 16,
        ChatDensity::Spacious => 24,
    }
}

/// A message row: the user's messages sit at the end, the system's are centered.
fn message_root(_theme: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    style()
        .flex_direction(if s.sender == ChatSender::User {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .align_items(Align::Start)
        .gap(gap(density(s)) / 2)
        .justify_content(if s.sender == ChatSender::System {
            Justify::Center
        } else {
            Justify::Start
        })
}

/// A message body: a filled bubble unless the bubble is a ghost, the system's divider rules, and the
/// corners that join a run of bubbles from the same sender.
fn message_body(_theme: &Theme, s: &Look<ChatMessageState>, _base: Style) -> Style {
    let d = density(s);
    let bubble = s.variant::<ChatBubbleVariant>().unwrap_or_else(|| {
        if s.variant::<ChatMessageVariant>() == Some(ChatMessageVariant::Plain) {
            ChatBubbleVariant::Ghost
        } else {
            ChatBubbleVariant::Filled
        }
    });
    let mut look = style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .max_width(pct(90))
        .gap(gap(d) / 4)
        .font_size(14)
        .line_height(20.0 / 14.0)
        .color(c::TEXT_PRIMARY);
    if s.sender != ChatSender::System && bubble == ChatBubbleVariant::Filled {
        look = look
            .padding_x(if d == ChatDensity::Spacious { 20 } else { 16 })
            .padding_y(if d == ChatDensity::Spacious { 16 } else { 12 })
            .border_radius(if d == ChatDensity::Compact {
                r::RADIUS_CONTAINER
            } else {
                r::RADIUS_CHAT
            })
            .background(c::NEUTRAL);
    }
    if s.sender == ChatSender::System
        && s.variant::<SystemMessageVariant>().unwrap_or_default() == SystemMessageVariant::Divider
    {
        look = look
            .border_top(1.0, color::BORDER)
            .border_bottom(1.0, color::BORDER)
            .padding_y(4);
    }
    let user = (s.sender == ChatSender::User)
        != (kivact::view::use_direction() == kivact::view::ReadingDirection::Rtl);
    if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::Last) {
        look = if user {
            look.border_top_right_radius(6.0)
        } else {
            look.border_top_left_radius(6.0)
        };
    }
    if matches!(s.group, ChatBubbleGroup::Middle | ChatBubbleGroup::First) {
        look = if user {
            look.border_bottom_right_radius(6.0)
        } else {
            look.border_bottom_left_radius(6.0)
        };
    }
    look
}

/// The author line: the small semibold secondary text.
fn author(_theme: &Theme, _s: &Look<ChatMessageState>, _base: Style) -> Style {
    style()
        .font_size(12)
        .line_height(20.0 / 12.0)
        .font_weight(600)
        .color(c::TEXT_SECONDARY)
}

/// The muted line of a message: the small secondary text.
fn muted(_theme: &Theme, _s: &Look<ChatMessageState>, _base: Style) -> Style {
    style()
        .font_size(12)
        .line_height(20.0 / 12.0)
        .color(c::TEXT_SECONDARY)
}

/// The thread's content column: it grows to fill the thread, padded and gapped by the density.
fn thread_content(_theme: &Theme, s: &Look<ChatThreadState>, _base: Style) -> Style {
    let g = gap(density(s));
    style()
        .flex_direction(FlexDirection::Column)
        .flex_grow(1.0)
        .padding_y(g)
        .padding_x(g.max(12))
        .gap(g)
}

/// The thread's composer: inset from the sides and the bottom by the density.
fn thread_composer(_theme: &Theme, s: &Look<ChatThreadState>, _base: Style) -> Style {
    style()
        .padding_x(gap(density(s)))
        .padding_bottom(gap(density(s)))
}

/// The composer's surface: a bordered column that takes the low shadow when it is elevated.
fn composer_surface(theme: &Theme, s: &Look<ChatComposerState>, _base: Style) -> Style {
    let compact = density(s) == ChatDensity::Compact;
    let look = style()
        .flex_direction(FlexDirection::Column)
        .min_width(0)
        .padding(if compact { 8 } else { 12 })
        .gap(4)
        .border_radius(if compact { 16.0 } else { 28.0 })
        .background(c::BG_SURFACE)
        .border(1.0, color::BORDER);
    if s.variant::<ComposerElevation>().unwrap_or_default() == ComposerElevation::Low {
        shadowed(look, shadows::low(theme.mode() == ThemeMode::Dark))
    } else {
        look
    }
}

/// The composer's input: no border and a transparent fill, so the surface frames it.
fn input(_theme: &Theme, _s: &Look<ChatComposerState>, _base: Style) -> Style {
    style()
        .padding(4)
        .border(0.0, color::BORDER)
        .background(Color::TRANSPARENT)
        .focus_visible(style().border(0.0, color::BORDER))
}

/// The composer's status line: the error or warning color, else the secondary text.
fn composer_status(_theme: &Theme, s: &Look<ChatComposerState>, _base: Style) -> Style {
    style().font_size(12).padding_x(8).color(match &s.status {
        Some(ComposerStatus::Error(_)) => c::TEXT_ERROR,
        Some(ComposerStatus::Warning(_)) => c::TEXT_WARNING,
        None => c::TEXT_SECONDARY,
    })
}

/// Install the kit's recipes of the chat group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ChatMessageTheme::default()
                .root(message_root)
                .body(message_body)
                .author(author)
                .metadata(muted)
                .time(muted)
                .status(muted),
        )
        .with(
            ChatThreadTheme::default()
                .content(thread_content)
                .composer(thread_composer),
        )
        .with(
            ChatComposerTheme::default()
                .surface(composer_surface)
                .input(input)
                .status(composer_status),
        )
}
