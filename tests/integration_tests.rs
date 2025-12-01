use template_rust::models::Todo;

#[test]
fn test_todo_creation() {
    let todo = Todo::new(
        "Test todo".to_string(),
        Some("Test description".to_string()),
    );

    assert_eq!(todo.title, "Test todo");
    assert_eq!(todo.description, Some("Test description".to_string()));
    assert!(!todo.completed);
    assert!(!todo.id.is_empty());
}

#[test]
fn test_todo_completion() {
    let mut todo = Todo::new("Test todo".to_string(), None);
    let original_updated_at = todo.updated_at;

    // Wait a moment to ensure timestamp change
    std::thread::sleep(std::time::Duration::from_millis(1));

    todo.complete();

    assert!(todo.completed);
    assert!(todo.updated_at > original_updated_at);
}

#[test]
fn test_todo_update() {
    let mut todo = Todo::new("Original title".to_string(), None);
    let original_updated_at = todo.updated_at;

    // Wait a moment to ensure timestamp change
    std::thread::sleep(std::time::Duration::from_millis(1));

    todo.update(
        Some("Updated title".to_string()),
        Some("New description".to_string()),
    );

    assert_eq!(todo.title, "Updated title");
    assert_eq!(todo.description, Some("New description".to_string()));
    assert!(todo.updated_at > original_updated_at);
}
