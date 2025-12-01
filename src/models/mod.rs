use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Todo item model
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Todo {
    /// Create a new todo item
    pub fn new(title: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            completed: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Mark todo as completed
    pub fn complete(&mut self) {
        self.completed = true;
        self.updated_at = Utc::now();
    }

    /// Mark todo as incomplete
    pub fn uncomplete(&mut self) {
        self.completed = false;
        self.updated_at = Utc::now();
    }

    /// Update todo title and description
    pub fn update(&mut self, title: Option<String>, description: Option<String>) {
        if let Some(title) = title {
            self.title = title;
        }
        if description.is_some() {
            self.description = description;
        }
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_todo() {
        let todo = Todo::new("Test".to_string(), Some("Description".to_string()));
        assert_eq!(todo.title, "Test");
        assert_eq!(todo.description, Some("Description".to_string()));
        assert!(!todo.completed);
        assert!(!todo.id.is_empty());
    }

    #[test]
    fn test_complete_todo() {
        let mut todo = Todo::new("Test".to_string(), None);
        todo.complete();
        assert!(todo.completed);
    }

    #[test]
    fn test_uncomplete_todo() {
        let mut todo = Todo::new("Test".to_string(), None);
        todo.complete();
        todo.uncomplete();
        assert!(!todo.completed);
    }

    #[test]
    fn test_update_todo() {
        let mut todo = Todo::new("Original".to_string(), None);
        todo.update(Some("Updated".to_string()), Some("New desc".to_string()));
        assert_eq!(todo.title, "Updated");
        assert_eq!(todo.description, Some("New desc".to_string()));
    }
}
