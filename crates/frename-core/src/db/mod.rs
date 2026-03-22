//! Application database: schema, migrations, and app state storage.
//!
//! Layout:
//! - [traits] — interfaces: `AppStateStore`, `Initializable`, `StoredTagNames`
//! - [logging] — decorator that adds logging around any store
//! - [fake_app_storage] — in-memory store for tests (implements both app state and tag names)
//! - [app_database] — SQLite implementation (no logging; wrap with logging for that)

mod app_database;
pub(crate) mod fake_app_storage;
mod logging;
mod migrations;
mod schema;
mod traits;

pub use app_database::AppDatabase;
pub use logging::LoggingAppStateStore;
pub use traits::{AppStateStore, Initializable, StoredTagStore, WindowGeometry};
