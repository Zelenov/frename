//! Trait for items that can be stored in an ordered collection with selection state.

use std::hash::Hash;

/// Integer type used for sort order. Matches DB storage (i64).
pub type OrderKey = i64;

/// Item that can be stored in an [OrderedCollection](super::OrderedCollection).
/// Provides identity, order (get/set), and selected/visible state for move-by-visible operations.
pub trait OrderedThing: Sized {
    /// Unique id type. Must be totally ordered for stable iteration and tie-breaking in the ordered index.
    type Id: Clone + Eq + Hash + Ord;

    /// Returns the unique id of this item.
    fn id(&self) -> Self::Id;

    /// Returns the current order key (used for sorted index).
    fn order(&self) -> OrderKey;

    /// Sets the order key. The collection will keep this in sync with its ordered index.
    fn set_order(&mut self, order: OrderKey);

    /// Whether this item is selected (e.g. "visible" in the UI). Used by [OrderedCollection::move_visible_before].
    fn is_selected(&self) -> bool;

    /// Sets the selected state.
    fn set_selected(&mut self, selected: bool);
}

/// Default entry type implementing [OrderedThing]. Use with any id type for [OrderedCollection].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderableEntry<Id> {
    id: Id,
    order: OrderKey,
    selected: bool,
}

impl<Id: Clone + Eq + std::hash::Hash + Ord> OrderableEntry<Id> {
    /// Creates an entry with the given id, order, and selected state.
    pub fn new(id: Id, order: OrderKey, selected: bool) -> Self {
        Self { id, order, selected }
    }

    /// Returns the id.
    pub fn id_ref(&self) -> &Id {
        &self.id
    }

    /// Returns the current order.
    pub fn order_value(&self) -> OrderKey {
        self.order
    }

    /// Returns whether this entry is selected.
    pub fn selected(&self) -> bool {
        self.selected
    }

    /// Sets the selected state.
    pub fn set_selected_value(&mut self, selected: bool) {
        self.selected = selected;
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

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}
