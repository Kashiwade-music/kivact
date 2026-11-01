//! Role: A style's stored draw closure, equal by identity.

use super::recorder::Draw;
use crate::geometry::Size;
use std::fmt;
use std::rc::Rc;

type PaintFn = dyn Fn(&mut Draw, Size);

/// A style's `paint(..)` callback; clones share it and compare by identity.
#[derive(Clone)]
pub(crate) struct LocalPaint(Rc<PaintFn>);

impl LocalPaint {
    pub(crate) fn new(f: impl Fn(&mut Draw, Size) + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub(crate) fn draw(&self, draw: &mut Draw, size: Size) {
        (self.0)(draw, size);
    }
}

impl fmt::Debug for LocalPaint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LocalPaint(..)")
    }
}

impl PartialEq for LocalPaint {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
