//! Role: Read responsive breakpoints from the viewport and measured containers.
use kivact::platform::window_size;
use kivact::view::NodeRef;
use kivact::{Value, memo};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Breakpoint {
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl Breakpoint {
    pub const fn px(self) -> f32 {
        match self {
            Self::Sm => 640.0,
            Self::Md => 768.0,
            Self::Lg => 1024.0,
            Self::Xl => 1280.0,
            Self::Xxl => 1536.0,
        }
    }
}

pub fn viewport_below(breakpoint: Breakpoint) -> Value<bool> {
    memo(move || window_size().width < breakpoint.px())
}

pub fn viewport_from(breakpoint: Breakpoint) -> Value<bool> {
    memo(move || window_size().width >= breakpoint.px())
}

pub fn is_compact() -> Value<bool> {
    memo(|| {
        let size = window_size();
        size.width < Breakpoint::Md.px() || size.height < 500.0
    })
}

pub fn container_below(node: NodeRef, width: f32) -> Value<bool> {
    let rect = node.rect();
    memo(move || rect.get().is_some_and(|rect| rect.width < width))
}
