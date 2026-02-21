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
    pub fn insert_before(&mut self, key: K, value: V, before_id: Option<&K>) {
        if self.by_id.contains_key(&key) {
            self.remove(&key).unwrap();
        }

        let (left_owned, right_owned) = match before_id {
            None => (None, self.iter().next().map(|k| k.0.clone())),
            Some(b) => {
                if self.by_id.contains_key(b) {
                    (
                        self.left_neighbor_of(b).map(|k| k.clone()),
                        Some(b.clone()),
                    )
                } else {
                    (None, None)
                }
            }
        };
        let new_order = self.order_between(left_owned.as_ref(), right_owned.as_ref());
        self.insert(key, value, new_order);
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

    fn order_between(
        &mut self,
        prev: Option<&K>,
        next: Option<&K>,
    ) -> OrderKey {

        // Use 0 for "no previous" to avoid i64 overflow in midpoint (next.checked_sub(i64::MIN) overflows).
        let prev_order = match prev {
            Some(id) => self.order_of.get(&id).copied().unwrap_or(0),
            None => 0,
        };
        let next_order = match next {
            Some(id) => self.order_of.get(&id).copied().unwrap_or(OrderKey::MAX),
            None => OrderKey::MAX,
        };

        match midpoint(prev_order, next_order) {
            Some(x) => x,
            None => {
                self.rebalance();
                self.order_between(prev, next)
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
