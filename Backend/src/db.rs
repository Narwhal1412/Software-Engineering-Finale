// This file is used to use a Database Driver to connect with mariadb for DAL

// Tells the compiler to auto-generate boilerplate code
#[derive(Clone)]
// In java this is basically data field, 
// In rust, they're located inside a struct block
pub struct AppState {
    pub db_name: String, 
}

// Same goes for Constructors, in Java they're methods
// In rust, they're located inside of impl(implmentation) block

impl AppState {
    // Associated Function(Doesn't have self parameter), therefore must be called via "::"
    // Self, stands for AppState own type
    // Or in java, an object constructor
    pub fn new() -> Self {
        Self {
            db_name: String::from("finale_db"),
        }
    }
}