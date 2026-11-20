// Bring AppState in from src/db.rs
use crate::db::AppState;

// DAL, this Layer only talks to the database.
// "&AppState" borrows a read-only reference instead of taking ownership.
pub async fn check_database_ping(state: &AppState) -> bool {
    // Placeholder until we add sqlx (later this runs `SELECT 1` on MariaDB).
    // No ":" on the last line means Rust returns this bool automatically.
    !state.db_name.is_empty()
}