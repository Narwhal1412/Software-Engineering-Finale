// 1. Imports
// :: Is used for navigating libraries and classes
// Just like java, import axum.routing.get, import axum.Router
use axum::{routing::get, Router};
use tower_http::cors::CorsLayer;

// Register the local modules,
// Rust ignores files until declared here:
// "mod db" -> src/db.rs
mod db;
mod controllers;
mod repositories;
mod services;

use db::AppState;

// The Server Engine
// Tokio is the async engine for rust, from a library.
#[tokio::main]
async fn main() {
   // Shared state, hold mariadb connection pool
   let state = AppState::new();

   // CORS middleware for local dev
   let cors = CorsLayer::permissive();

   // 4. The URL Router
   // In Rust, URLs must be explicitly mapped to controller functions:
   // When a GET request hits /api/health, run controllers/health.rs -> health_check.
   let app = Router::new()
               .route("/api/health", get(controllers::health::health_check)).layer(cors).with_state(state);

   // 5. Opening The Network Port
    // Must bind to 0.0.0.0 inside Docker so outside containers/host can reach it.
    // .await goes with async: whenever an operation touches the network (binding port,
    // querying database), put .await at the end.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Rust backend started at 0.0.0.0:8000");

    // Start the infinite loop listening for HTTP requests forever.
    // .unwrap() basically crashes the program and prints out the error if one occurs.
    axum::serve(listener, app).await.unwrap();
}
