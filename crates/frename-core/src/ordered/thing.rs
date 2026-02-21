//! Trait for items that can be stored in an ordered collection (order only; selection is handled by the caller).

use std::hash::Hash;

/// Integer type used for sort order. Matches DB storage (i64).
pub type OrderKey = i64;

/// Item that can be stored in an [OrderedCollection](super::OrderedCollection).
/// Provides identity and order (get/set). Selection/visibility is not part of the collection; callers filter when needed.
pub trait OrderedThing: Sized {
    /// Unique id type. Must be totally ordered for stable iteration and tie-breaking in the ordered index.
    type Id: Clone + Eq + Hash + Ord;

    /// Returns the unique id of this item.
    fn id(&self) -> Self::Id;

    /// Returns the current order key (used for sorted index).
    fn order(&self) -> OrderKey;

    /// Sets the order key. The collection will keep this in sync with its ordered index.
    fn set_order(&mut self, order: OrderKey);
}

/// Default entry type implementing [OrderedThing]. Use with any id type for [OrderedCollection].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderableEntry<Id> {
    id: Id,
    order: OrderKey,
}

impl<Id: Clone + Eq + std::hash::Hash + Ord> OrderableEntry<Id> {
    /// Creates an entry with the given id and order.
    pub fn new(id: Id, order: OrderKey) -> Self {
        Self { id, order }
    }

    /// Returns the id.
    pub fn id_ref(&self) -> &Id {
        &self.id
    }

    /// Returns the current order.
    pub fn order_value(&self) -> OrderKey {
        self.order
    }
}

impl<Id: Clone + Eq + std::hash::Hash + Ord> OrderedThing for OrderableEntry<Id> {
    type Id = Id;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn order(&self) -> OrderKey {
        self.order
    }

    fn set_order(&mut self, order: OrderKey) {
        self.order = order;
    }
}
