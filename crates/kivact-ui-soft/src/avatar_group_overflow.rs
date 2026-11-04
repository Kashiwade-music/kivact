//! Role: Show the number of people omitted from a facepile.
use crate::avatar::{avatar_focus, group_frame};
use crate::avatar_group::AvatarGroupContext;
use crate::avatar_options::{AvatarShape, AvatarSize};
use crate::tokens;
use kivact::Value;
use kivact::control::{Node, NodeBuilder, node, text};
use kivact::i18n::message_with;
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Cursor, Direction, Justify, pct, style};
use kivact::view::{
    IntoText, IntoValue, IntoView, ReadingDirection, Role, View, current_direction, try_context,
};

/// A static or actionable count of omitted avatars.
pub struct AvatarGroupOverflow {
    root: Node,
    count: Value<i64>,
    content: Option<Value<String>>,
    press: Option<Box<dyn Fn()>>,
}

/// Show the number of people outside the visible group.
pub fn avatar_group_overflow(count: impl IntoValue<i64>) -> AvatarGroupOverflow {
    let count = count.into_value();
    AvatarGroupOverflow {
        root: node().role(Role::Image).name(message_with(
            "soft.avatar_group.overflow",
            "{count} more",
            move || [("count", count.get().max(0).to_string())],
        )),
        count,
        content: None,
        press: None,
    }
}

impl AvatarGroupOverflow {
    /// Override the visible count text.
    pub fn content(mut self, content: impl IntoText) -> Self {
        self.content = Some(content.into_text_value());
        self
    }
    /// Open the full list when pressed.
    pub fn on_press(mut self, action: impl Fn() + 'static) -> Self {
        self.root = self.root.role(Role::Button);
        self.press = Some(Box::new(action));
        self
    }
}

impl NodeBuilder for AvatarGroupOverflow {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for AvatarGroupOverflow {
    fn into_view(self) -> View {
        let Self {
            mut root,
            count,
            content,
            press,
        } = self;
        let group = try_context::<AvatarGroupContext>();
        let grouped = group.is_some();
        let rtl = current_direction() == ReadingDirection::Rtl;
        let (side, shape) = group
            .as_ref()
            .map_or((AvatarSize::Md.side(), AvatarShape::Circle), |cx| {
                (cx.side, cx.shape)
            });
        let interactive = press.is_some();
        if let Some(press) = press {
            root = root.part(part(|input| input).on_press(press));
            root = if let Some(group) = group {
                group.register(root)
            } else {
                root.focusable(true)
            };
        }
        let label = text(move || {
            content.map_or_else(|| format!("+{}", count.get().max(0)), |value| value.get())
        });
        let surface = node()
            .style(move || {
                let p = tokens();
                style()
                    .height(pct(100))
                    .min_width(side)
                    .padding_x(p.spacing.s2)
                    .radius(shape.radius())
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .background(p.colors.neutral)
                    .color(p.colors.text_secondary)
                    .font_size((side * 0.35).max(p.typography.supporting.size_px))
                    .weight(p.typography.weight_medium)
                    .text_wrap(false)
            })
            .child(label);
        root.base_style(move || {
            let p = tokens();
            let look = style()
                .height(side + 4.0)
                .min_width(side + 4.0)
                .shrink(0)
                .radius(shape.radius())
                .background(p.colors.bg_surface)
                .border(2, p.colors.bg_surface);
            let look = if grouped {
                group_frame(look, side, rtl)
            } else {
                look
            };
            if interactive {
                avatar_focus(look.cursor(Cursor::Pointer))
            } else {
                look
            }
        })
        .child(surface)
        .into_view()
    }
}
