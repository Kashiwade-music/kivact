//! Role: Prefix sums of a virtual list's row heights (a Fenwick tree).

/// The heights of a list's rows by index, with O(log N) prefix sums,
/// point updates and position lookups.
pub(super) struct Heights {
    /// The row heights.
    rows: Vec<f64>,
    /// The Fenwick tree over `rows` (1-based).
    tree: Vec<f64>,
}

impl Heights {
    /// The tree of `rows` (O(N)).
    pub(super) fn build(rows: Vec<f64>) -> Self {
        let n = rows.len();
        let mut tree = vec![0.0; n + 1];
        for i in 1..=n {
            tree[i] += rows[i - 1];
            let parent = i + i.isolate_lowest_one();
            if parent <= n {
                let value = tree[i];
                tree[parent] += value;
            }
        }
        Heights { rows, tree }
    }

    /// An empty list.
    pub(super) fn empty() -> Self {
        Heights {
            rows: Vec::new(),
            tree: vec![0.0],
        }
    }

    /// The height of the rows before `index`.
    pub(super) fn prefix(&self, index: usize) -> f64 {
        let mut sum = 0.0;
        let mut i = index.min(self.rows.len());
        while i > 0 {
            sum += self.tree[i];
            i &= i - 1;
        }
        sum
    }

    /// The height of every row.
    pub(super) fn total(&self) -> f64 {
        self.prefix(self.rows.len())
    }

    /// Set the height of row `index`.
    pub(super) fn set(&mut self, index: usize, height: f64) {
        let Some(old) = self.rows.get_mut(index) else {
            return;
        };
        let delta = height - *old;
        *old = height;
        let mut i = index + 1;
        while i < self.tree.len() {
            self.tree[i] += delta;
            i += i.isolate_lowest_one();
        }
    }

    /// The index of the row that contains position `y` (the last row
    /// for a `y` past the end, 0 for an empty list).
    pub(super) fn index_at(&self, y: f64) -> usize {
        let n = self.rows.len();
        if n == 0 {
            return 0;
        }
        let mut pos = 0;
        let mut rest = y;
        let mut step = n.next_power_of_two();
        while step > 0 {
            let next = pos + step;
            if next <= n && self.tree[next] <= rest {
                pos = next;
                rest -= self.tree[next];
            }
            step >>= 1;
        }
        pos.min(n - 1)
    }
}

#[cfg(test)]
mod tests {
    //! Role: Prefix sums and lookups match a plain sum.

    use super::Heights;

    #[test]
    fn prefix_set_and_index_at_match_a_plain_sum() {
        let mut h = Heights::build(vec![10.0, 20.0, 30.0, 40.0, 50.0]);
        assert_eq!(h.prefix(3), 60.0);
        assert_eq!(h.total(), 150.0);
        assert_eq!(h.index_at(0.0), 0);
        assert_eq!(h.index_at(29.9), 1);
        assert_eq!(h.index_at(30.0), 2);
        assert_eq!(h.index_at(1000.0), 4);
        h.set(1, 5.0);
        assert_eq!(h.prefix(2), 15.0);
        assert_eq!(h.index_at(15.0), 2);
    }
}
