//! Template Rust - Todo App Example
//!
//! This is a template Rust project featuring a todo application with SQLite database
//! and terminal user interface (TUI).

pub mod database;
pub mod models;
pub mod tui;

pub use models::*;

/// Application result type
pub type Result<T> = anyhow::Result<T>;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Database file path
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "todo.db".to_string(),
        }
    }
}
