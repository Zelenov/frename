//! Application database: schema, migrations, and app state storage.
//!
//! Layout:
//! - [traits] — interfaces: `AppStateStore`, `Initializable`
//! - [logging] — decorator that adds logging around any store
//! - [empty_store] — no-op store for tests
//! - [app_database] — SQLite implementation (no logging; wrap with logging for that)

mod app_database;
mod empty_store;
mod logging;
mod migrations;
mod schema;
mod traits;

pub use app_database::AppDatabase;
pub use logging::LoggingAppStateStore;
pub use traits::{AppStateStore, Initializable};
