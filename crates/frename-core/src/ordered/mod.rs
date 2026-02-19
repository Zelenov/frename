//! Generic ordered collection with unique keys, selected state, and rebalance on no-space.
//!
//! Uses `BTreeSet<(OrderKey, Id)>` so duplicate order values are allowed (tie-breaker is Id).
//! Rebalance is only needed when there is no integer between two neighbors.

mod collection;
mod thing;

pub use collection::OrderedCollection;
pub use thing::{OrderableEntry, OrderKey, OrderedThing};
