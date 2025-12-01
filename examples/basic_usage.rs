use template_rust::{database::TodoDatabase, models::Todo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let db = TodoDatabase::new("example.db").await?;

    // Create some todos
    let todo1 = Todo::new(
        "Learn Rust".to_string(),
        Some("Study ownership and borrowing".to_string()),
    );
    let todo2 = Todo::new("Build a CLI app".to_string(), None);
    let todo3 = Todo::new(
        "Write tests".to_string(),
        Some("Unit and integration tests".to_string()),
    );

    // Save todos to database
    db.create_todo(&todo1).await?;
    db.create_todo(&todo2).await?;
    db.create_todo(&todo3).await?;

    // List all todos
    println!("All todos:");
    let todos = db.get_all_todos().await?;
    for todo in &todos {
        let status = if todo.completed { "✓" } else { "○" };
        println!("  {} {}", status, todo.title);
        if let Some(description) = &todo.description {
            println!("    {}", description);
        }
    }

    // Complete the first todo
    let mut todo = todos[0].clone();
    todo.complete();
    db.update_todo(&todo).await?;

    println!("\nAfter completing first todo:");
    let updated_todos = db.get_all_todos().await?;
    for todo in &updated_todos {
        let status = if todo.completed { "✓" } else { "○" };
        println!("  {} {}", status, todo.title);
    }

    // Get only pending todos
    println!("\nPending todos:");
    let pending_todos = db.get_todos_by_status(false).await?;
    for todo in &pending_todos {
        println!("  ○ {}", todo.title);
    }

    Ok(())
}
