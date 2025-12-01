use crate::models::Todo;
use crate::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Database service for todo operations
#[derive(Debug, Clone)]
pub struct TodoDatabase {
    pool: SqlitePool,
}

impl TodoDatabase {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self> {
        // Handle special cases for SQLite URL format
        let url = match database_url {
            ":memory:" => "sqlite::memory:".to_string(),
            path if path.starts_with("sqlite://") => path.to_string(),
            path => {
                // Create parent directory if needed for file databases
                if let Some(parent) = std::path::Path::new(path).parent() {
                    std::fs::create_dir_all(parent)?;
                }
                format!("sqlite://{}?mode=rwc", path)
            }
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;

        let db = Self { pool };
        db.migrate().await?;
        Ok(db)
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                completed BOOLEAN NOT NULL DEFAULT FALSE,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get all todos
    pub async fn get_all_todos(&self) -> Result<Vec<Todo>> {
        let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        Ok(todos)
    }

    /// Get a todo by ID
    pub async fn get_todo(&self, id: &str) -> Result<Option<Todo>> {
        let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(todo)
    }

    /// Create a new todo
    pub async fn create_todo(&self, todo: &Todo) -> Result<()> {
        sqlx::query(
            "INSERT INTO todos (id, title, description, completed, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&todo.id)
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(todo.completed)
        .bind(todo.created_at.to_rfc3339())
        .bind(todo.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update a todo
    pub async fn update_todo(&self, todo: &Todo) -> Result<()> {
        sqlx::query(
            "UPDATE todos SET title = ?, description = ?, completed = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(todo.completed)
        .bind(todo.updated_at.to_rfc3339())
        .bind(&todo.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Delete a todo
    pub async fn delete_todo(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM todos WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get todos by completion status
    pub async fn get_todos_by_status(&self, completed: bool) -> Result<Vec<Todo>> {
        let todos = sqlx::query_as::<_, Todo>(
            "SELECT * FROM todos WHERE completed = ? ORDER BY created_at DESC",
        )
        .bind(completed)
        .fetch_all(&self.pool)
        .await?;
        Ok(todos)
    }
}
