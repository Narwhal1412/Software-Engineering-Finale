use serde::Serialize;
use crate::{db::AppState, repositories};


// Tells serde to auto-generate JSON conversion code for this struct,
// So field names automatically becomes JSON keys("status",...)
// And Rust type map to JSON types(String -> string, bool -> true/false, i32 -> number)
#[derive(Serialize)]
// DTO(Data Transfer Object), to be converted into another format, 
// In this case, JSON
pub struct HealthResponse {
    pub status: String,
    pub database_connected: bool,
    pub sample_tax_total: i32, // In rust, it's a good practice to have , at the end.
}

// Function format that takes and returns a 32-bit integer
fn calculate_tax(price: i32) -> i32 {
    // In Rust, the final line automatically returns without ";"
    price + 100
}

// BLL: Calls The DAL (repositories), applies business logic, and builds the DTO
pub async fn get_system_health(state: &AppState) -> HealthResponse {
    let db_ok = repositories::health::check_database_ping(state).await;
    let sample_total = calculate_tax(2000);

    HealthResponse {
        status: String::from("Backend container is alive!!"),
        database_connected: db_ok,
        sample_tax_total: sample_total,
    }
}