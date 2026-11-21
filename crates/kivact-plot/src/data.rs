//! Role: The numeric arrays a plot draws, shared by reference.

use std::fmt;
use std::sync::Arc;

/// A numeric array shared by reference. Cloning is cheap. Two values are
/// equal when they share the array or hold the same numbers bit for bit, so a
/// reactive value rebuilt from the same data does not redraw.
///
/// The series constructors take `impl Into<PlotData>`, so a `Vec<f64>`, an
/// array, a slice or an iterator can be passed directly.
#[derive(Clone)]
pub struct PlotData {
    values: Arc<[f64]>,
}

impl PlotData {
    /// The array `values`.
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values: values.into(),
        }
    }

    /// The values in order.
    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    /// The number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether there are no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl From<Vec<f64>> for PlotData {
    fn from(values: Vec<f64>) -> Self {
        Self::new(values)
    }
}

impl From<&[f64]> for PlotData {
    /// A copy of `values`.
    fn from(values: &[f64]) -> Self {
        Self::new(values.to_vec())
    }
}

impl<const N: usize> From<[f64; N]> for PlotData {
    fn from(values: [f64; N]) -> Self {
        Self::new(values.to_vec())
    }
}

impl FromIterator<f64> for PlotData {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl PartialEq for PlotData {
    /// Equal when both share one array, or hold the same numbers bit for bit.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.values, &other.values)
            || (self.values.len() == other.values.len()
                && self
                    .values
                    .iter()
                    .zip(other.values.iter())
                    .all(|(a, b)| a.to_bits() == b.to_bits()))
    }
}

impl fmt::Debug for PlotData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlotData(len = {})", self.len())
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the equality, the conversions and the debug text of plot data.

    use super::*;

    #[test]
    fn data_are_equal_when_bit_for_bit_equal() {
        let a = PlotData::new(vec![0.0, -0.0]);
        assert_eq!(a, a.clone());
        assert_eq!(a, PlotData::new(vec![0.0, -0.0]));
        assert_ne!(a, PlotData::new(vec![0.0, 0.0]));
        assert_eq!(format!("{a:?}"), "PlotData(len = 2)");
    }

    #[test]
    fn arrays_slices_vectors_and_iterators_convert() {
        let from_array = PlotData::from([1.0, 2.0]);
        let from_slice = PlotData::from(&[1.0, 2.0][..]);
        let from_vec: PlotData = vec![1.0, 2.0].into();
        let from_iter: PlotData = (1..=2).map(f64::from).collect();
        for data in [from_array, from_slice, from_vec, from_iter] {
            assert_eq!(data.as_slice(), &[1.0, 2.0]);
            assert!(!data.is_empty());
        }
        assert!(PlotData::from(Vec::<f64>::new()).is_empty());
    }
}
