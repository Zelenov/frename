//! Ordered dictionary: key -> value with a stable order. Order is stored and managed internally.
//!
//! **Public API:** [insert], [insert_before], [get_order], [iter] (and get, remove, len).
//! Iteration yields `(id, value, order)` tuples.

use std::collections::{BTreeSet, HashMap};
use std::ops::Bound;

use super::thing::OrderKey;

/// Returns a key strictly between `a` and `b`, or `None` if no integer fits (triggers rebalance).
fn midpoint(a: OrderKey, b: OrderKey) -> Option<OrderKey> {
    let gap = b.checked_sub(a)?;
    if gap <= 1 {
        None
    } else {
        a.checked_add(gap / 2)
    }
}

/// Ordered collection: like a dictionary (key -> value) with a stable order. Order is internal; callers use ids.
#[derive(Clone, Debug)]
pub struct OrderedCollection<K, V> {
    by_id: HashMap<K, V>,
    order_of: HashMap<K, OrderKey>,
    ordered: BTreeSet<(OrderKey, K)>,
}

impl<K, V> Default for OrderedCollection<K, V>
where
    K: Clone + Eq + std::hash::Hash + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> OrderedCollection<K, V>
where
    K: Clone + Eq + std::hash::Hash + Ord,
{
    /// Creates an empty collection.
    pub fn new() -> Self {
        Self {
            by_id: HashMap::new(),
            order_of: HashMap::new(),
            ordered: BTreeSet::new(),
        }
    }

    /// Inserts (key, value) with the given order. Replaces existing entry with the same key.
    pub fn insert(&mut self, key: K, value: V, order: OrderKey) {
        if let Some(&old_order) = self.order_of.get(&key) {
            self.ordered.remove(&(old_order, key.clone()));
        }
        self.by_id.insert(key.clone(), value);
        self.order_of.insert(key.clone(), order);
        self.ordered.insert((order, key));
    }

    /// Inserts (key, value) immediately before `before_id`. If `before_id` — inserts before the first element.
    /// If empty — inserts in the middle.
    /// If `key` is already in the collection, this moves it to the new position (remove then insert before).
    /// Returns `true` if the collection was rebalanced (order keys were respread); callers should persist display order when saving.
    pub fn insert_before(&mut self, key: K, value: V, before_id: Option<&K>) -> bool {
        if self.by_id.contains_key(&key) {
            self.remove(&key).unwrap();
        }

        let (left_owned, right_owned) = match before_id {
            None => (None, self.iter().next().map(|k| k.0.clone())),
            Some(b) => {
                if self.by_id.contains_key(b) {
                    (self.left_neighbor_of(b).cloned(), Some(b.clone()))
                } else {
                    (None, None)
                }
            }
        };
        let (new_order, rebalanced) = self.order_between(left_owned.as_ref(), right_owned.as_ref());
        self.insert(key, value, new_order);
        rebalanced
    }
    /// Returns the order for the given key, if present.
    pub fn get_order(&self, id: &K) -> Option<OrderKey> {
        self.order_of.get(id).copied()
    }

    /// Iterator over all entries in order: `(id, value, order)`.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V, OrderKey)> {
        self.ordered.iter().filter_map(move |(order, id)| {
            let v = self.by_id.get(id)?;
            Some((id, v, *order))
        })
    }

    /// Returns a reference to the value for the key.
    pub fn get(&self, id: &K) -> Option<&V> {
        self.by_id.get(id)
    }

    /// Returns a mutable reference to the value for the key.
    pub fn get_mut(&mut self, id: &K) -> Option<&mut V> {
        self.by_id.get_mut(id)
    }

    /// Removes the entry by key. Returns the value if present.
    pub fn remove(&mut self, id: &K) -> Option<V> {
        let value = self.by_id.remove(id)?;
        let order = self.order_of.remove(id)?;
        self.ordered.remove(&(order, id.clone()));
        Some(value)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Rebalance: spread all items across most of the i64 range so there is space before the first and after the last.
    /// Uses range [1, i64::MAX - 1] and divides it into (n + 1) gaps, placing n items at the first n gap boundaries.
    fn rebalance(&mut self) {
        let ids: Vec<K> = self.ordered.iter().map(|(_, id)| id.clone()).collect();
        let n = ids.len();
        if n == 0 {
            return;
        }
        const RANGE_MIN: OrderKey = 1;
        const RANGE_MAX: OrderKey = i64::MAX - 1;
        let range_size = RANGE_MAX.saturating_sub(RANGE_MIN);
        let gap = range_size / (n as OrderKey + 1);
        self.ordered.clear();
        for (i, id) in ids.into_iter().enumerate() {
            let new_order = RANGE_MIN.saturating_add(gap.saturating_mul((i as OrderKey) + 1));
            self.order_of.insert(id.clone(), new_order);
            self.ordered.insert((new_order, id));
        }
    }

    fn left_neighbor_of(&self, id: &K) -> Option<&K> {
        let current_order = match self.order_of.get(id) {
            Some(&o) => o,
            None => return None,
        };
        let key = (current_order, id.clone());
        let prev = self
            .ordered
            .range((Bound::Unbounded, Bound::Excluded(&key)))
            .next_back()
            .map(|(_, id)| id);
        prev
    }

    fn right_neighbor_of(&self, id: &K) -> Option<&K> {
        let current_order = match self.order_of.get(id) {
            Some(&o) => o,
            None => return None,
        };
        let key = (current_order, id.clone());
        self.ordered
            .range((Bound::Excluded(&key), Bound::Unbounded))
            .next()
            .map(|(_, id)| id)
    }

    /// Inserts (key, value) immediately after `after_id`.
    /// `insert_after(key, value, None)` inserts at position 0 (before the current first element).
    /// `insert_after(key, value, Some(anchor))` inserts immediately after `anchor`; if `anchor` is last, appends at end.
    /// If `key` is already in the collection it is removed first (moved to the new position).
    /// Returns `true` if the collection was rebalanced.
    pub fn insert_after(&mut self, key: K, value: V, after_id: Option<&K>) -> bool {
        if self.by_id.contains_key(&key) {
            self.remove(&key).unwrap();
        }

        let (left_owned, right_owned) = match after_id {
            None => {
                // Insert at very beginning: before the current first element.
                (None, self.iter().next().map(|k| k.0.clone()))
            }
            Some(a) => {
                if self.by_id.contains_key(a) {
                    let left = Some(a.clone());
                    let right = self.right_neighbor_of(a).cloned();
                    (left, right)
                } else {
                    (None, None)
                }
            }
        };
        let (new_order, rebalanced) = self.order_between(left_owned.as_ref(), right_owned.as_ref());
        self.insert(key, value, new_order);
        rebalanced
    }

    /// Returns `true` if the common elements (keys present in both `self` and `other`) appear in
    /// the same relative order in both collections.
    ///
    /// Elements that exist only in one collection are ignored. An empty intersection is always `true`.
    pub fn is_same_order_as<V2>(&self, other: &OrderedCollection<K, V2>) -> bool {
        let other_keys: std::collections::HashSet<&K> = other.iter().map(|(k, _, _)| k).collect();
        let self_keys: std::collections::HashSet<&K> = self.iter().map(|(k, _, _)| k).collect();

        let self_common: Vec<&K> = self
            .iter()
            .filter(|(k, _, _)| other_keys.contains(k))
            .map(|(k, _, _)| k)
            .collect();
        let other_common: Vec<&K> = other
            .iter()
            .filter(|(k, _, _)| self_keys.contains(k))
            .map(|(k, _, _)| k)
            .collect();

        self_common == other_common
    }

    /// Reorders the common elements in `self` to match their relative order in `other`.
    ///
    /// Elements that exist only in `self` keep their positions relative to each other;
    /// common elements are rearranged by inserting out-of-order ones after the last correctly
    /// placed one ("insert the incorrect element after the correct one").
    ///
    /// # Example
    /// ```text
    /// self:  [x, A, y, B, z, C]   other: [A, C, B]
    /// after: [x, A, C, y, B, z]
    /// ```
    pub fn sync_order_from<V2>(&mut self, other: &OrderedCollection<K, V2>) {
        let other_keys: std::collections::HashSet<K> =
            other.iter().map(|(k, _, _)| k.clone()).collect();

        // Common keys in other's order.
        let common_in_other_order: Vec<K> = other
            .iter()
            .filter(|(k, _, _)| self.by_id.contains_key(*k))
            .map(|(k, _, _)| k.clone())
            .collect();

        let mut last_anchor: Option<K> = None;

        for key in &common_in_other_order {
            // Re-read self's common subsequence (may have changed from previous moves).
            let common_in_self: Vec<K> = self
                .iter()
                .filter(|(k, _, _)| other_keys.contains(*k))
                .map(|(k, _, _)| k.clone())
                .collect();

            let expected_next: Option<K> = match &last_anchor {
                None => common_in_self.first().cloned(),
                Some(anchor) => {
                    let pos = common_in_self.iter().position(|k| k == anchor);
                    pos.and_then(|p| common_in_self.get(p + 1)).cloned()
                }
            };

            if expected_next.as_ref() == Some(key) {
                // Already in correct relative position.
                last_anchor = Some(key.clone());
            } else {
                // Move key to immediately after last_anchor.
                let value = self.remove(key).expect("key in both collections");
                self.insert_after(key.clone(), value, last_anchor.as_ref());
                last_anchor = Some(key.clone());
            }
        }
    }

    /// Returns (order_key, rebalanced). rebalanced is true when no integer fit between prev and next and rebalance was run.
    fn order_between(&mut self, prev: Option<&K>, next: Option<&K>) -> (OrderKey, bool) {
        // Use 0 for "no previous" to avoid i64 overflow in midpoint (next.checked_sub(i64::MIN) overflows).
        let prev_order = match prev {
            Some(id) => self.order_of.get(id).copied().unwrap_or(0),
            None => 0,
        };
        let next_order = match next {
            Some(id) => self.order_of.get(id).copied().unwrap_or(OrderKey::MAX),
            None => OrderKey::MAX,
        };

        match midpoint(prev_order, next_order) {
            Some(x) => (x, false),
            None => {
                self.rebalance();
                let (o, _) = self.order_between(prev, next);
                (o, true)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_iter() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn iter_yields_id_value_order() {
        let mut c: OrderedCollection<u32, String> = OrderedCollection::new();
        c.insert(1, "a".into(), 10);
        c.insert(2, "b".into(), 20);
        let triples: Vec<_> = c.iter().map(|(k, v, o)| (*k, v.as_str(), o)).collect();
        assert_eq!(triples.len(), 2);
        assert_eq!(triples[0], (1, "a", 10));
        assert_eq!(triples[1], (2, "b", 20));
    }

    #[test]
    fn get_order_returns_internal_order() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        assert_eq!(c.get_order(&1), Some(100));
        assert_eq!(c.get_order(&2), Some(200));
    }

    #[test]
    fn move_before_rebalance() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 1);
        c.insert(2, (), 2);
        c.insert(3, (), 3);
        c.insert_before(3, (), Some(&1));
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [3, 1, 2]);
    }

    #[test]
    fn insert_before_and_move_before() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        c.insert_before(3, (), Some(&2));
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [1, 3, 2]);
        if let Some(v) = c.remove(&1) {
            c.insert_before(1, v, Some(&2));
        }
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [3, 1, 2]);
    }

    #[test]
    fn insert_after_none_goes_to_front() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        c.insert_after(3, (), None);
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [3, 1, 2]);
    }

    #[test]
    fn insert_after_anchor_places_after_anchor() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        c.insert(3, (), 300);
        c.insert_after(4, (), Some(&2));
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [1, 2, 4, 3]);
    }

    #[test]
    fn insert_after_last_appends_at_end() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        c.insert_after(3, (), Some(&2));
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [1, 2, 3]);
    }

    #[test]
    fn insert_after_moves_existing_element() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 100);
        c.insert(2, (), 200);
        c.insert(3, (), 300);
        // Move 1 after 3 (to end)
        c.insert_after(1, (), Some(&3));
        let ids: Vec<u32> = c.iter().map(|(id, _, _)| *id).collect();
        assert_eq!(ids, [2, 3, 1]);
    }

    // ── is_same_order_as ────────────────────────────────────────────────────

    #[test]
    fn same_order_identical_common() {
        // this: A B C (plus non-common elements)  other: A B C D
        // Common A B C are in the same order → true
        let mut this: OrderedCollection<&str, ()> = OrderedCollection::new();
        this.insert("x", (), 10);
        this.insert("A", (), 20);
        this.insert("y", (), 30);
        this.insert("B", (), 40);
        this.insert("C", (), 50);

        let mut other: OrderedCollection<&str, ()> = OrderedCollection::new();
        other.insert("A", (), 1);
        other.insert("B", (), 2);
        other.insert("C", (), 3);
        other.insert("D", (), 4);

        assert!(this.is_same_order_as(&other));
    }

    #[test]
    fn same_order_different_common_order() {
        // this: A B C   other: A C B  → false
        let mut this: OrderedCollection<&str, ()> = OrderedCollection::new();
        this.insert("A", (), 10);
        this.insert("B", (), 20);
        this.insert("C", (), 30);

        let mut other: OrderedCollection<&str, ()> = OrderedCollection::new();
        other.insert("A", (), 1);
        other.insert("C", (), 2);
        other.insert("B", (), 3);

        assert!(!this.is_same_order_as(&other));
    }

    #[test]
    fn same_order_empty_intersection() {
        let mut this: OrderedCollection<&str, ()> = OrderedCollection::new();
        this.insert("A", (), 10);

        let mut other: OrderedCollection<&str, ()> = OrderedCollection::new();
        other.insert("B", (), 1);

        assert!(this.is_same_order_as(&other));
    }

    // ── sync_order_from ─────────────────────────────────────────────────────

    #[test]
    fn sync_already_correct_order() {
        // this: [x, A, y, B, z, C]  other: [A, B, C, D]  → no change
        let mut this: OrderedCollection<&str, ()> = OrderedCollection::new();
        this.insert("x", (), 5);
        this.insert("A", (), 10);
        this.insert("y", (), 15);
        this.insert("B", (), 20);
        this.insert("z", (), 25);
        this.insert("C", (), 30);

        let mut other: OrderedCollection<&str, ()> = OrderedCollection::new();
        other.insert("A", (), 1);
        other.insert("B", (), 2);
        other.insert("C", (), 3);
        other.insert("D", (), 4);

        this.sync_order_from(&other);

        let ids: Vec<&str> = this.iter().map(|(k, _, _)| *k).collect();
        // A, B, C still in same relative order; x, y, z also in their original relative positions
        let common: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|k| ["A", "B", "C"].contains(k))
            .collect();
        assert_eq!(common, ["A", "B", "C"]);
    }

    #[test]
    fn sync_reorders_a_c_b() {
        // this: [x, A, y, B, z, C]  other: [A, C, B]  → common become A C B
        let mut this: OrderedCollection<&str, ()> = OrderedCollection::new();
        this.insert("x", (), 5);
        this.insert("A", (), 10);
        this.insert("y", (), 15);
        this.insert("B", (), 20);
        this.insert("z", (), 25);
        this.insert("C", (), 30);

        let mut other: OrderedCollection<&str, ()> = OrderedCollection::new();
        other.insert("A", (), 1);
        other.insert("C", (), 2);
        other.insert("B", (), 3);

        this.sync_order_from(&other);

        let common: Vec<&str> = this
            .iter()
            .map(|(k, _, _)| *k)
            .filter(|k| ["A", "B", "C"].contains(k))
            .collect();
        assert_eq!(common, ["A", "C", "B"]);
    }

    #[test]
    fn sync_reorders_c_a_b() {
        // this: [x, A, y, B, z, C]  other: [C, A, B]  → common become C A B
        let mut this: OrderedCollection<&str, ()> = OrderedCollection::new();
        this.insert("x", (), 5);
        this.insert("A", (), 10);
        this.insert("y", (), 15);
        this.insert("B", (), 20);
        this.insert("z", (), 25);
        this.insert("C", (), 30);

        let mut other: OrderedCollection<&str, ()> = OrderedCollection::new();
        other.insert("C", (), 1);
        other.insert("A", (), 2);
        other.insert("B", (), 3);

        this.sync_order_from(&other);

        let common: Vec<&str> = this
            .iter()
            .map(|(k, _, _)| *k)
            .filter(|k| ["A", "B", "C"].contains(k))
            .collect();
        assert_eq!(common, ["C", "A", "B"]);
    }

    #[test]
    fn sync_preserves_non_common_relative_order() {
        // Non-common elements should keep their relative order among themselves.
        let mut this: OrderedCollection<u32, ()> = OrderedCollection::new();
        this.insert(0, (), 5);
        this.insert(10, (), 10); // common
        this.insert(1, (), 15);
        this.insert(20, (), 20); // common
        this.insert(2, (), 25);
        this.insert(30, (), 30); // common

        let mut other: OrderedCollection<u32, ()> = OrderedCollection::new();
        other.insert(30, (), 1);
        other.insert(10, (), 2);
        other.insert(20, (), 3);

        this.sync_order_from(&other);

        let all: Vec<u32> = this.iter().map(|(k, _, _)| *k).collect();
        // non-common [0, 1, 2] must still appear in the same relative order
        let non_common: Vec<u32> = all
            .iter()
            .copied()
            .filter(|k| [0u32, 1, 2].contains(k))
            .collect();
        assert_eq!(non_common, [0, 1, 2]);
        // common [10, 20, 30] must be in other's order: 30, 10, 20
        let common: Vec<u32> = all
            .iter()
            .copied()
            .filter(|k| [10u32, 20, 30].contains(k))
            .collect();
        assert_eq!(common, [30, 10, 20]);
    }

    #[test]
    fn rebalance_single_item_move_before_self_no_op() {
        let mut c: OrderedCollection<u32, ()> = OrderedCollection::new();
        c.insert(1, (), 1);
        let (moved_id, before_id) = (1, 1);
        if moved_id != before_id {
            if let Some(v) = c.remove(&moved_id) {
                c.insert_before(moved_id, v, Some(&before_id));
            }
        }
        assert_eq!(c.len(), 1);
        assert_eq!(c.iter().map(|(id, _, _)| *id).collect::<Vec<_>>(), [1]);
    }
}
