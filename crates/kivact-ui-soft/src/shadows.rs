//! Role: Resolve Astryx Neutral elevation and inset shadows.
use kivact::draw::Shadow;
use kivact::style::{rgb, rgba};

/// Elevation shadows in front-to-back drawing order.
#[derive(Clone, Debug, PartialEq)]
pub struct Shadows {
    pub low: Vec<Shadow>,
    pub med: Vec<Shadow>,
    pub high: Vec<Shadow>,
    pub inset_hover: Shadow,
    pub inset_selected: Shadow,
    pub inset_success: Shadow,
    pub inset_warning: Shadow,
    pub inset_error: Shadow,
    pub input_hover_ring: Shadow,
    pub input_focus_ring: Shadow,
}

impl Shadows {
    pub(super) fn new(dark: bool) -> Self {
        let outside = |y, blur, alpha| Shadow::new(0, y, blur, rgba(0, alpha));
        let inset = |spread, color| Shadow::new(0, 0, 0, color).spread(spread).inset();
        let (low, med, high) = if dark {
            (
                vec![
                    outside(2, 4, 64.0 / 255.0),
                    outside(4, 8, 102.0 / 255.0),
                    inset(1, rgba(0xffffff, 20.0 / 255.0)),
                ],
                vec![
                    outside(2, 4, 89.0 / 255.0),
                    outside(4, 12, 128.0 / 255.0),
                    inset(1, rgba(0xffffff, 31.0 / 255.0)),
                ],
                vec![
                    outside(4, 6, 128.0 / 255.0),
                    outside(12, 24, 179.0 / 255.0),
                    inset(1, rgba(0xffffff, 38.0 / 255.0)),
                ],
            )
        } else {
            (
                vec![outside(2, 4, 13.0 / 255.0), outside(4, 8, 26.0 / 255.0)],
                vec![outside(2, 4, 13.0 / 255.0), outside(4, 12, 26.0 / 255.0)],
                vec![outside(4, 6, 26.0 / 255.0), outside(12, 24, 38.0 / 255.0)],
            )
        };
        Self {
            low,
            med,
            high,
            inset_hover: inset(2, rgba(0x0074e2, 77.0 / 255.0)),
            inset_selected: inset(2, rgba(0x0074e2, 128.0 / 255.0)),
            inset_success: inset(2, rgba(0x2f7d33, 77.0 / 255.0)),
            inset_warning: inset(2, rgba(0xf8d36a, 77.0 / 255.0)),
            inset_error: inset(2, rgba(0xde4745, 77.0 / 255.0)),
            input_hover_ring: inset(2, rgba(if dark { 0x525252 } else { 0xd4d4d4 }, 0.3)),
            input_focus_ring: inset(2, rgb(if dark { 0x262626 } else { 0xf1f1f1 })),
        }
    }
}
