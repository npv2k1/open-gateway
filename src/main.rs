use clap::{Parser, Subcommand};
use template_rust::{database::TodoDatabase, models::Todo, tui::App};

/// A simple todo application with SQLite and TUI
#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A terminal todo application")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Database file path
    #[arg(short, long, default_value = "todo.db")]
    database: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive TUI
    Tui,
    /// List all todos
    List {
        /// Show only completed todos
        #[arg(short, long)]
        completed: bool,
        /// Show only pending todos
        #[arg(short, long)]
        pending: bool,
    },
    /// Add a new todo
    Add {
        /// Todo title
        title: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Complete a todo by ID
    Complete {
        /// Todo ID
        id: String,
    },
    /// Delete a todo by ID
    Delete {
        /// Todo ID
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let db = TodoDatabase::new(&cli.database).await?;

    match cli.command {
        Some(Commands::Tui) | None => {
            // Default to TUI mode
            let mut app = App::new(db);
            app.run().await?;
        }
        Some(Commands::List { completed, pending }) => {
            let todos = if completed {
                db.get_todos_by_status(true).await?
            } else if pending {
                db.get_todos_by_status(false).await?
            } else {
                db.get_all_todos().await?
            };

            if todos.is_empty() {
                println!("No todos found.");
            } else {
                for todo in todos {
                    let status = if todo.completed { "✓" } else { "○" };
                    println!("{} {} - {}", status, todo.title, todo.id);
                    if let Some(description) = &todo.description {
                        println!("   {}", description);
                    }
                }
            }
        }
        Some(Commands::Add { title, description }) => {
            let todo = Todo::new(title, description);
            db.create_todo(&todo).await?;
            println!("Todo added: {}", todo.id);
        }
        Some(Commands::Complete { id }) => {
            if let Some(mut todo) = db.get_todo(&id).await? {
                todo.complete();
                db.update_todo(&todo).await?;
                println!("Todo completed: {}", todo.title);
            } else {
                eprintln!("Todo not found: {}", id);
            }
        }
        Some(Commands::Delete { id }) => {
            if let Some(todo) = db.get_todo(&id).await? {
                db.delete_todo(&id).await?;
                println!("Todo deleted: {}", todo.title);
            } else {
                eprintln!("Todo not found: {}", id);
            }
        }
    }

    Ok(())
}
