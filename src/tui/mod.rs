use crate::database::TodoDatabase;
use crate::models::Todo;
use crate::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::time::Duration;

/// Application state
pub struct App {
    db: TodoDatabase,
    todos: Vec<Todo>,
    selected: ListState,
    input: String,
    input_mode: InputMode,
    status_message: String,
    filter: Filter,
}

#[derive(Debug, Clone)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, Clone)]
pub enum Filter {
    All,
    Completed,
    Pending,
}

impl App {
    pub fn new(db: TodoDatabase) -> Self {
        let mut selected = ListState::default();
        selected.select(Some(0));

        Self {
            db,
            todos: Vec::new(),
            selected,
            input: String::new(),
            input_mode: InputMode::Normal,
            status_message: "Welcome to Todo App! Press 'h' for help.".to_string(),
            filter: Filter::All,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        self.refresh_todos().await?;

        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match self.input_mode {
                            InputMode::Normal => {
                                if self.handle_normal_input(key.code).await? {
                                    break;
                                }
                            }
                            InputMode::Editing => {
                                if self.handle_editing_input(key.code).await? {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_normal_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('h') => {
                self.status_message = "Commands: q=quit, n=new todo, d=delete, c=toggle complete, a=all, p=pending, f=finished, ↑↓=navigate".to_string();
            }
            KeyCode::Char('n') => {
                self.input_mode = InputMode::Editing;
                self.input.clear();
                self.status_message = "Enter new todo (ESC to cancel, Enter to save):".to_string();
            }
            KeyCode::Char('d') => {
                if let Some(index) = self.selected.selected() {
                    if index < self.todos.len() {
                        let todo = &self.todos[index];
                        self.db.delete_todo(&todo.id).await?;
                        self.refresh_todos().await?;
                        self.status_message = "Todo deleted!".to_string();
                    }
                }
            }
            KeyCode::Char('c') => {
                if let Some(index) = self.selected.selected() {
                    if index < self.todos.len() {
                        let mut todo = self.todos[index].clone();
                        if todo.completed {
                            todo.uncomplete();
                        } else {
                            todo.complete();
                        }
                        self.db.update_todo(&todo).await?;
                        self.refresh_todos().await?;
                        self.status_message = if todo.completed {
                            "Todo marked as completed!".to_string()
                        } else {
                            "Todo marked as pending!".to_string()
                        };
                    }
                }
            }
            KeyCode::Char('a') => {
                self.filter = Filter::All;
                self.refresh_todos().await?;
                self.status_message = "Showing all todos".to_string();
            }
            KeyCode::Char('p') => {
                self.filter = Filter::Pending;
                self.refresh_todos().await?;
                self.status_message = "Showing pending todos".to_string();
            }
            KeyCode::Char('f') => {
                self.filter = Filter::Completed;
                self.refresh_todos().await?;
                self.status_message = "Showing completed todos".to_string();
            }
            KeyCode::Down => {
                let i = match self.selected.selected() {
                    Some(i) => {
                        if i >= self.todos.len().saturating_sub(1) {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.selected.select(Some(i));
            }
            KeyCode::Up => {
                let i = match self.selected.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.todos.len().saturating_sub(1)
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.selected.select(Some(i));
            }
            _ => {}
        }
        Ok(false)
    }

    async fn handle_editing_input(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    let todo = Todo::new(self.input.trim().to_string(), None);
                    self.db.create_todo(&todo).await?;
                    self.input.clear();
                    self.input_mode = InputMode::Normal;
                    self.refresh_todos().await?;
                    self.status_message = "Todo added!".to_string();
                }
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Esc => {
                self.input.clear();
                self.input_mode = InputMode::Normal;
                self.status_message = "Cancelled".to_string();
            }
            _ => {}
        }
        Ok(false)
    }

    async fn refresh_todos(&mut self) -> Result<()> {
        self.todos = match self.filter {
            Filter::All => self.db.get_all_todos().await?,
            Filter::Completed => self.db.get_todos_by_status(true).await?,
            Filter::Pending => self.db.get_todos_by_status(false).await?,
        };

        // Adjust selection if needed
        if self.todos.is_empty() {
            self.selected.select(None);
        } else if let Some(selected) = self.selected.selected() {
            if selected >= self.todos.len() {
                self.selected.select(Some(self.todos.len() - 1));
            }
        } else {
            self.selected.select(Some(0));
        }

        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.size());

        // Title
        let title = Paragraph::new("📝 Todo App")
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Todo list
        let todos: Vec<ListItem> = self
            .todos
            .iter()
            .map(|todo| {
                let status = if todo.completed { "✓" } else { "○" };
                let style = if todo.completed {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(Color::White)
                };

                let content = format!("{} {}", status, todo.title);
                ListItem::new(content).style(style)
            })
            .collect();

        let filter_text = match self.filter {
            Filter::All => "All",
            Filter::Completed => "Completed",
            Filter::Pending => "Pending",
        };

        let todos_list = List::new(todos)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Todos ({})", filter_text)),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        f.render_stateful_widget(todos_list, chunks[1], &mut self.selected);

        // Status/Input bar
        let status_text = match self.input_mode {
            InputMode::Normal => self.status_message.clone(),
            InputMode::Editing => format!("New todo: {}", self.input),
        };

        let status = Paragraph::new(status_text)
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL).title("Status"));

        f.render_widget(status, chunks[2]);
    }
}
