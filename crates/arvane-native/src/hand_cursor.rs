//! Role: Open and closed hand cursor bitmaps for platforms without them.

/// The open hand mask (`.` transparent, `#` black, `o` white).
const OPEN: [&str; 16] = [
    "......##........",
    "...###oo###.....",
    "..#oo#oo#oo#....",
    "..#oo#oo#oo###..",
    "..#oo#oo#oo#oo#.",
    "..#oo#oo#oo#oo#.",
    "..#oo#oo#oo#oo#.",
    "..#ooooooooooo#.",
    "###ooooooooooo#.",
    "#ooooooooooooo#.",
    "#ooooooooooooo#.",
    ".#oooooooooooo#.",
    "..#ooooooooooo#.",
    "...#ooooooooo#..",
    "....#ooooooo#...",
    ".....#######....",
];

/// The closed hand mask (`.` transparent, `#` black, `o` white).
const CLOSED: [&str; 16] = [
    "................",
    "................",
    "................",
    "...##.##.##.##..",
    "..#oo#oo#oo#oo#.",
    "..#ooooooooooo#.",
    "###ooooooooooo#.",
    "#ooooooooooooo#.",
    "#ooooooooooooo#.",
    ".#oooooooooooo#.",
    "..#ooooooooooo#.",
    "...#ooooooooo#..",
    "....#ooooooo#...",
    ".....#######....",
    "................",
    "................",
];

const BLACK: [u8; 4] = [0, 0, 0, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];
const CLEAR: [u8; 4] = [0, 0, 0, 0];

/// One of the two hand cursors kivact draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Hand {
    /// The open hand, shown over grabbable content.
    Open,
    /// The closed hand, shown while content is being dragged.
    Closed,
}

/// The pixel factor of the hand bitmaps at a display scale: the 16 × 16 masks
/// drawn 1, 2, 3 or 4 times as large. A NaN scale gives 1.
pub(crate) fn hand_factor(scale: f64) -> u32 {
    if scale.is_nan() {
        1
    } else {
        scale.round().clamp(1.0, 4.0) as u32
    }
}

/// The RGBA pixels (row-major, top row first), the side and the hot spot of a
/// hand at `factor` (1 to 4): each mask pixel is a `factor` × `factor` block.
pub(crate) fn hand_rgba(hand: Hand, factor: u32) -> (Vec<u8>, u32, (u32, u32)) {
    let rows = match hand {
        Hand::Open => &OPEN,
        Hand::Closed => &CLOSED,
    };
    let block = factor as usize;
    let side = 16 * factor;
    let stride = side as usize;
    let mut rgba = Vec::with_capacity(stride * stride * 4);
    for y in 0..stride {
        let row = rows[y / block].as_bytes();
        for x in 0..stride {
            let color = match row[x / block] {
                b'#' => BLACK,
                b'o' => WHITE,
                _ => CLEAR,
            };
            rgba.extend_from_slice(&color);
        }
    }
    (rgba, side, (8 * factor, 8 * factor))
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the hand cursor bitmaps.
    use super::*;

    #[test]
    fn every_mask_row_is_sixteen_transparent_black_or_white_pixels() {
        for row in OPEN.iter().chain(CLOSED.iter()) {
            assert_eq!(row.len(), 16, "row {row:?}");
            assert!(
                row.chars().all(|c| matches!(c, '.' | '#' | 'o')),
                "row {row:?}"
            );
        }
    }

    #[test]
    fn the_factor_is_the_rounded_scale_from_one_to_four() {
        assert_eq!(hand_factor(1.0), 1);
        assert_eq!(hand_factor(1.25), 1);
        assert_eq!(hand_factor(1.5), 2);
        assert_eq!(hand_factor(2.0), 2);
        assert_eq!(hand_factor(3.0), 3);
        assert_eq!(hand_factor(0.5), 1);
        assert_eq!(hand_factor(5.0), 4);
        assert_eq!(hand_factor(f64::NAN), 1);
    }

    #[test]
    fn the_open_hand_at_one_times_has_its_hot_spot_and_a_black_pixel() {
        let (rgba, side, hot) = hand_rgba(Hand::Open, 1);
        assert_eq!((side, hot), (16, (8, 8)));
        assert_eq!(rgba.len(), 16 * 16 * 4);
        let at = 6 * 4;
        assert_eq!(&rgba[at..at + 4], &[0, 0, 0, 255]);
    }

    #[test]
    fn the_open_hand_at_two_times_draws_each_mask_pixel_as_a_block() {
        let (rgba, side, hot) = hand_rgba(Hand::Open, 2);
        assert_eq!((side, hot), (32, (16, 16)));
        let at = (32 + 13) * 4;
        assert_eq!(&rgba[at..at + 4], &[0, 0, 0, 255]);
    }

    #[test]
    fn the_closed_hand_at_two_times_is_thirty_two_pixels_square() {
        let (rgba, side, _) = hand_rgba(Hand::Closed, 2);
        assert_eq!(side, 32);
        assert_eq!(rgba.len(), 32 * 32 * 4);
    }
}
