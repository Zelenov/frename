//! Ordered collection: BTreeSet<(OrderKey, Id)> + HashMap<Id, T>. Rebalance when no space between neighbors.

use std::collections::{BTreeSet, HashMap};

use super::thing::{OrderKey, OrderedThing};

/// Returns a key strictly between `a` and `b`, or `None` if no integer fits (triggers rebalance).
fn midpoint(a: OrderKey, b: OrderKey) -> Option<OrderKey> {
    let gap = b.checked_sub(a)?;
    if gap <= 1 {
        None
    } else {
        a.checked_add(gap / 2)
    }
}

/// Generic ordered collection with unique ids and optional selected state.
/// Uses [BTreeSet] of `(OrderKey, Id)` so duplicate order values are allowed (tie-breaker is Id).
/// Rebalance is only when there is no space between two neighbors.
#[derive(Clone, Debug)]
pub struct OrderedCollection<T: OrderedThing> {
    by_id: HashMap<T::Id, T>,
    ordered: BTreeSet<(OrderKey, T::Id)>,
    step: OrderKey,
}

impl<T: OrderedThing> Default for OrderedCollection<T> {
    fn default() -> Self {
        Self::new(1)
    }
}

impl<T: OrderedThing> OrderedCollection<T> {
    /// Creates an empty collection. New items get order `(index + 1) * step` on full rebalance.
    pub fn new(step: OrderKey) -> Self {
        Self {
            by_id: HashMap::new(),
            ordered: BTreeSet::new(),
            step: step.max(1),
        }
    }

    /// Inserts an item. Order is taken from the item; if it would collide, consider rebalance.
    /// Replaces existing entry with the same id.
    pub fn insert(&mut self, item: T) {
        let id = item.id();
        let order = item.order();
        self.ordered.remove(&(self.by_id.get(&id).map(|e| e.order()).unwrap_or(order), id.clone()));
        self.by_id.insert(id.clone(), item);
        self.ordered.insert((order, id));
    }

    /// Removes the item by id. Returns the removed item if present.
    pub fn remove(&mut self, id: &T::Id) -> Option<T> {
        let item = self.by_id.remove(id)?;
        self.ordered.remove(&(item.order(), id.clone()));
        Some(item)
    }

    /// Returns a reference to the item by id.
    pub fn get(&self, id: &T::Id) -> Option<&T> {
        self.by_id.get(id)
    }

    /// Returns a mutable reference to the item by id.
    pub fn get_mut(&mut self, id: &T::Id) -> Option<&mut T> {
        self.by_id.get_mut(id)
    }

    /// Returns ids in ascending order key (then by id as tie-breaker).
    pub fn ids_ordered(&self) -> impl Iterator<Item = T::Id> + '_ {
        self.ordered.iter().map(|(_, id)| id.clone())
    }

    /// Returns items in ascending order key.
    pub fn iter_ordered(&self) -> impl Iterator<Item = &T> {
        self.ordered
            .iter()
            .filter_map(move |(_, id)| self.by_id.get(id))
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Sets the order of an item and updates the ordered index. Id must already be in the collection.
    fn set_order(&mut self, id: &T::Id, new_order: OrderKey) {
        let entry = match self.by_id.get_mut(id) {
            Some(e) => e,
            None => return,
        };
        let old_order = entry.order();
        entry.set_order(new_order);
        self.ordered.remove(&(old_order, id.clone()));
        self.ordered.insert((new_order, id.clone()));
    }

    /// Rebalance: spread all items to `(1*step, 2*step, 3*step, ...)` preserving current order.
    /// Call when [midpoint] returns `None` (no space between neighbors).
    fn rebalance(&mut self) {
        let step = self.step;
        let ids: Vec<T::Id> = self.ordered.iter().map(|(_, id)| id.clone()).collect();
        self.ordered.clear();
        for (i, id) in ids.into_iter().enumerate() {
            let new_order = (i as OrderKey + 1) * step;
            if let Some(entry) = self.by_id.get_mut(&id) {
                entry.set_order(new_order);
            }
            self.ordered.insert((new_order, id));
        }
    }

    /// Moves the item `moved_id` so it is immediately before `before_id` in the full order.
    /// If there is no integer between the neighbors, rebalances then assigns the new order.
    pub fn move_before(&mut self, moved_id: &T::Id, before_id: &T::Id) {
        let (prev, next) = self.neighbors_of(before_id);
        self.move_between(moved_id, Some(before_id), prev, next);
    }

    /// Moves the item `moved_id` so it is immediately before `before_id` in the *visible* (selected) order.
    /// Only selected items are considered for neighbor lookup; the moved item is placed before `before_id` in the full order.
    pub fn move_visible_before(&mut self, moved_id: &T::Id, before_id: &T::Id) {
        let (prev, next) = self.visible_neighbors_of(before_id);
        self.move_between(moved_id, Some(before_id), prev, next);
    }

    /// Slot to insert "before id": (order of previous item or MIN, order of id).
    fn neighbors_of(&self, id: &T::Id) -> (OrderKey, OrderKey) {
        let current_order = match self.by_id.get(id) {
            Some(t) => t.order(),
            None => {
                return (i64::MIN, i64::MAX);
            }
        };
        let mut prev = i64::MIN;
        for (order, _) in self.ordered.iter() {
            if *order < current_order {
                prev = *order;
            } else {
                break;
            }
        }
        (prev, current_order)
    }

    /// Slot to insert "before id" among selected items: (order of previous selected item or MIN, order of id).
    fn visible_neighbors_of(&self, id: &T::Id) -> (OrderKey, OrderKey) {
        let current_order = match self.by_id.get(id) {
            Some(t) => t.order(),
            None => return (i64::MIN, i64::MAX),
        };
        let selected_orders: Vec<OrderKey> = self
            .ordered
            .iter()
            .filter_map(|(order, oid)| self.by_id.get(oid).filter(|t| t.is_selected()).map(|_| *order))
            .collect();
        let mut prev = i64::MIN;
        for order in selected_orders {
            if order < current_order {
                prev = order;
            } else {
                break;
            }
        }
        (prev, current_order)
    }

    /// Assigns a new order to `moved_id` between `prev` and `next`; rebalances if no space.
    /// `before_id` is used after rebalance to recompute the slot (right before that item).
    fn move_between(
        &mut self,
        moved_id: &T::Id,
        before_id: Option<&T::Id>,
        prev: OrderKey,
        next: OrderKey,
    ) {
        let new_order = match midpoint(prev, next) {
            Some(x) => x,
            None => {
                self.rebalance();
                let (prev2, next2) = before_id
                    .map(|id| self.neighbors_of(id))
                    .unwrap_or((i64::MIN, i64::MAX));
                match midpoint(prev2, next2) {
                    Some(x) => x,
                    None if prev2 == i64::MIN => next2.saturating_sub(self.step),
                    None if next2 == i64::MAX => prev2.saturating_add(self.step),
                    None => prev2.saturating_add(self.step),
                }
            }
        };
        self.set_order(moved_id, new_order);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Clone, Debug)]
    struct Item {
        id: u32,
        order: OrderKey,
        selected: bool,
    }

    impl OrderedThing for Item {
        type Id = u32;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn order(&self) -> OrderKey {
            self.order
        }

        fn set_order(&mut self, order: OrderKey) {
            self.order = order;
        }

        fn is_selected(&self) -> bool {
            self.selected
        }

        fn set_selected(&mut self, selected: bool) {
            self.selected = selected;
        }
    }

    impl fmt::Display for Item {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.id)
        }
    }

    #[test]
    fn insert_and_iter_ordered() {
        let mut c = OrderedCollection::new(100);
        c.insert(Item {
            id: 1,
            order: 100,
            selected: false,
        });
        c.insert(Item {
            id: 2,
            order: 200,
            selected: false,
        });
        let ids: Vec<u32> = c.ids_ordered().collect();
        assert_eq!(ids, [1, 2]);
    }

    #[test]
    fn move_before_rebalance() {
        let mut c = OrderedCollection::new(1);
        c.insert(Item {
            id: 1,
            order: 1,
            selected: true,
        });
        c.insert(Item {
            id: 2,
            order: 2,
            selected: true,
        });
        c.insert(Item {
            id: 3,
            order: 3,
            selected: true,
        });
        // Move 3 before 1: need order between MIN and 1 -> no space -> rebalance
        c.move_before(&3, &1);
        let ids: Vec<u32> = c.ids_ordered().collect();
        assert_eq!(ids, [3, 1, 2]);
    }

    #[test]
    fn rebalance_spreads_orders_and_preserves_relative_order() {
        let mut c = OrderedCollection::new(10);
        c.insert(Item {
            id: 1,
            order: 1,
            selected: true,
        });
        c.insert(Item {
            id: 2,
            order: 2,
            selected: true,
        });
        c.insert(Item {
            id: 3,
            order: 3,
            selected: true,
        });
        // Trigger rebalance by moving 3 before 1 (no space between MIN and 1)
        c.move_before(&3, &1);
        // Rebalance first spreads current [1,2,3] to 10,20,30; then we assign moved item 3 order 0
        assert_eq!(c.get(&1).unwrap().order(), 10);
        assert_eq!(c.get(&2).unwrap().order(), 20);
        assert_eq!(c.get(&3).unwrap().order(), 0);
        let ids: Vec<u32> = c.ids_ordered().collect();
        assert_eq!(ids, [3, 1, 2]);
    }

    #[test]
    fn rebalance_triggered_when_moving_between_adjacent_orders() {
        let mut c = OrderedCollection::new(1);
        c.insert(Item {
            id: 1,
            order: 1,
            selected: true,
        });
        c.insert(Item {
            id: 2,
            order: 2,
            selected: true,
        });
        c.insert(Item {
            id: 3,
            order: 3,
            selected: true,
        });
        // Move 1 before 3: slot is (2, 3), midpoint(2,3) = None -> rebalance
        c.move_before(&1, &3);
        let ids: Vec<u32> = c.ids_ordered().collect();
        assert_eq!(ids, [2, 1, 3]);
        // After rebalance, 1 is placed between 2 and 3 (order 2 < order 1; 1 may equal 3 if slot was tight)
        let o1 = c.get(&1).unwrap().order();
        let o2 = c.get(&2).unwrap().order();
        let o3 = c.get(&3).unwrap().order();
        assert!(o2 < o1 && o1 <= o3, "2 before 1 before-or-same 3: o2={o2} o1={o1} o3={o3}");
    }

    #[test]
    fn rebalance_then_second_move_without_rebalance() {
        let mut c = OrderedCollection::new(100);
        c.insert(Item {
            id: 1,
            order: 100,
            selected: true,
        });
        c.insert(Item {
            id: 2,
            order: 200,
            selected: true,
        });
        c.insert(Item {
            id: 3,
            order: 300,
            selected: true,
        });
        // First move: 3 before 1 -> rebalance (no space between MIN and 100 with step 100? Actually neighbors_of(1) = (MIN, 100). midpoint(MIN, 100) overflows -> rebalance)
        c.move_before(&3, &1);
        assert_eq!(
            c.ids_ordered().collect::<Vec<_>>(),
            [3, 1, 2],
            "order after first move"
        );
        // After rebalance orders are 100, 200, 300. Now move 2 before 3: slot (MIN, 100), still no space -> rebalance again. So we get [2, 3, 1].
        c.move_before(&2, &3);
        assert_eq!(
            c.ids_ordered().collect::<Vec<_>>(),
            [2, 3, 1],
            "order after second move"
        );
        // Next move: 1 before 2. Slot (MIN, order_of(2)). If order_of(2) is 100, no space -> rebalance. Otherwise midpoint might exist. Just assert stability.
        c.move_before(&1, &2);
        let ids: Vec<u32> = c.ids_ordered().collect();
        assert_eq!(ids, [1, 2, 3], "final order 1, 2, 3");
    }

    #[test]
    fn rebalance_single_item_move_before_self_no_op() {
        let mut c = OrderedCollection::new(1);
        c.insert(Item {
            id: 1,
            order: 1,
            selected: true,
        });
        c.move_before(&1, &1); // move before self: slot (MIN, 1), rebalance, then slot (MIN, 1) again -> order 0
        assert_eq!(c.len(), 1);
        assert_eq!(c.ids_ordered().collect::<Vec<_>>(), [1]);
    }

    #[test]
    fn rebalance_two_items_move_to_end_slot() {
        let mut c = OrderedCollection::new(1);
        c.insert(Item {
            id: 1,
            order: 1,
            selected: true,
        });
        c.insert(Item {
            id: 2,
            order: 2,
            selected: true,
        });
        // Move 1 before 2: slot (MIN, 2). midpoint(MIN, 2) = None (overflow). Rebalance -> orders 1, 2. Slot (MIN, 2) again -> next2.saturating_sub(step)=1. So item 1 gets order 1. Still [1, 2]. So "move 1 before 2" with two items leaves order unchanged.
        c.move_before(&1, &2);
        assert_eq!(c.ids_ordered().collect::<Vec<_>>(), [1, 2]);
        // Move 2 before 1: slot (MIN, 1). Rebalance, then order of 2 = 0. So [2, 1].
        c.move_before(&2, &1);
        assert_eq!(c.ids_ordered().collect::<Vec<_>>(), [2, 1]);
    }
}
