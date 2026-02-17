//! Application database: schema, migrations, and app state storage interface.

mod app_state;
mod migrations;
mod schema;

pub use app_state::{AppDatabase, AppStateStore, EmptyAppStateStore};
