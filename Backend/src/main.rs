// 1. Imports
// :: Is used for navigating libraries and classes
// Just like java, import axum.routing.get, import axum.Router
use axum::{routing::get, Router};

// Function format, that returns an integer
fn calculate_tax(price: i32) -> i32 {
   // In rust, the final line without ";", automatically returns.
   price + 100
}

// 2. Endpoint function
// In rust, an endpoint is just a normal function that returns a response
async fn health_check() -> String {
   let sample_total = calculate_tax(1000);
   // JSON format
   // "\" are used to tell the compiler to treat the next character as a normal one.
   // format!("total: {}", variable_in_placeholder), then returns a string
   // {{}}, tells rust to treat it as {} for JSON
   format!("{{\"status\": \"Backend container is alive!!\", \"sample_tax_total\": {} }}", sample_total)
}

// get version endpoint
async fn get_version() -> String {
   // String object, basically new String("Version 1.0") in java,
   // "version 1.0".to_string() also works.
   String::from("Version 1.0")
}


// 3. The Server Engine
// Tokio is the async engine for rust, from a library.
#[tokio::main]
async fn main() {
   // 4. The URL Router
   // In Rust, URLs must be explicitly mapped to functions:
   // When a GET request hit /api/health, run the health_check function.
   let app = Router::new().route("/api/health", get(health_check))
                          .route("/api/version", get(get_version));
    
   // 5. Opening The Network Port
   // Must bind to 0.0.0.0 inside Docker so outside containers/host can reach it
   // .await goes with async, whenever an operation touches the network(binding port, querying database),
   // put .await at the end.
   let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
   println!("Rust backend listening on 0.0.0.0");
   // Start the infinite loop listening for HTTP requests forever.
   // .unwrap basically crashes the program, and then print out the error,
   // If there was ever one.
   axum::serve(listener, app).await.unwrap();
}
