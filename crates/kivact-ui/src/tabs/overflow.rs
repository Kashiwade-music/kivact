//! Role: How a tab strip treats tabs wider than its box.
//! It also counts how many tabs fit before the "More" button.

/// What a tab strip does with tabs wider than its box.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TabOverflow {
    /// The tabs that do not fit move into a "More tabs" menu at the end of the strip.
    Menu,
    /// The strip scrolls along its axis; the selected tab scrolls into view.
    Scroll,
}

/// The number of leading tabs of `widths` that fit `strip` with `gap`
/// between them, leaving room for a `more` button when not all fit.
pub(crate) fn visible_count(strip: f64, widths: &[f64], gap: f64, more: f64) -> usize {
    let total: f64 = widths.iter().sum::<f64>() + gap * widths.len().saturating_sub(1) as f64;
    if total <= strip {
        return widths.len();
    }
    let room = strip - more - gap;
    let mut used = 0.0;
    for (index, width) in widths.iter().enumerate() {
        let next = used + width + if index == 0 { 0.0 } else { gap };
        if next > room {
            return index;
        }
        used = next;
    }
    widths.len()
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the tab overflow count.

    use super::*;

    #[test]
    fn all_tabs_fit_or_the_rest_leave_room_for_more() {
        assert_eq!(visible_count(300.0, &[100.0, 100.0], 4.0, 32.0), 2);
        assert_eq!(visible_count(250.0, &[100.0, 100.0, 100.0], 4.0, 32.0), 2);
        assert_eq!(visible_count(120.0, &[100.0, 100.0, 100.0], 4.0, 32.0), 0);
    }
}
