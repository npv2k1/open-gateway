//! TUI Monitor module
//!
//! This module provides a terminal user interface for monitoring:
//! - Metrics display
//! - Health check status
//! - Configuration overview
//! - Route information

use crate::config::GatewayConfig;
use crate::health::HealthChecker;
use crate::metrics::GatewayMetrics;
use crate::proxy::ProxyRoute;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use tokio::time::Duration;

/// Tab selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Overview,
    Routes,
    Config,
    Help,
}

impl Tab {
    fn titles() -> Vec<&'static str> {
        vec!["Overview", "Routes", "Config", "Help"]
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Overview,
            1 => Tab::Routes,
            2 => Tab::Config,
            3 => Tab::Help,
            _ => Tab::Overview,
        }
    }

    fn index(&self) -> usize {
        match self {
            Tab::Overview => 0,
            Tab::Routes => 1,
            Tab::Config => 2,
            Tab::Help => 3,
        }
    }
}

/// TUI Monitor application
pub struct MonitorApp {
    config: GatewayConfig,
    metrics: Arc<GatewayMetrics>,
    health: Arc<HealthChecker>,
    routes: Vec<ProxyRoute>,
    current_tab: Tab,
    route_list_state: ListState,
    should_quit: bool,
}

impl MonitorApp {
    /// Create a new monitor application
    pub fn new(
        config: GatewayConfig,
        metrics: Arc<GatewayMetrics>,
        health: Arc<HealthChecker>,
        routes: Vec<ProxyRoute>,
    ) -> Self {
        let mut route_list_state = ListState::default();
        if !routes.is_empty() {
            route_list_state.select(Some(0));
        }

        Self {
            config,
            metrics,
            health,
            routes,
            current_tab: Tab::Overview,
            route_list_state,
            should_quit: false,
        }
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app(&mut terminal).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_input(key.code);
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Tab | KeyCode::Right => {
                let next_index = (self.current_tab.index() + 1) % 4;
                self.current_tab = Tab::from_index(next_index);
            }
            KeyCode::BackTab | KeyCode::Left => {
                let prev_index = if self.current_tab.index() == 0 {
                    3
                } else {
                    self.current_tab.index() - 1
                };
                self.current_tab = Tab::from_index(prev_index);
            }
            KeyCode::Char('1') => self.current_tab = Tab::Overview,
            KeyCode::Char('2') => self.current_tab = Tab::Routes,
            KeyCode::Char('3') => self.current_tab = Tab::Config,
            KeyCode::Char('4') | KeyCode::Char('h') => self.current_tab = Tab::Help,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_tab == Tab::Routes && !self.routes.is_empty() {
                    let i = match self.route_list_state.selected() {
                        Some(i) => {
                            if i >= self.routes.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.route_list_state.select(Some(i));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_tab == Tab::Routes && !self.routes.is_empty() {
                    let i = match self.route_list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.routes.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.route_list_state.select(Some(i));
                }
            }
            _ => {}
        }
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Tabs
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Status bar
            ])
        .split(f.area());

        self.render_title(f, chunks[0]);
        self.render_tabs(f, chunks[1]);

        match self.current_tab {
            Tab::Overview => self.render_overview(f, chunks[2]),
            Tab::Routes => self.render_routes(f, chunks[2]),
            Tab::Config => self.render_config(f, chunks[2]),
            Tab::Help => self.render_help(f, chunks[2]),
        }

        self.render_status_bar(f, chunks[3]);
    }

    fn render_title(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("🚀 Open Gateway Monitor")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    fn render_tabs(&self, f: &mut Frame, area: Rect) {
        let titles: Vec<Line> = Tab::titles()
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let style = if i == self.current_tab.index() {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Line::from(Span::styled(format!(" {} ", t), style))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL).title("Tabs"))
            .select(self.current_tab.index())
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(tabs, area);
    }

    fn render_overview(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left side: Metrics
        let metrics = self.metrics.snapshot();
        let health_response = self.health.liveness();

        let metrics_text = vec![
            Line::from(vec![
                Span::styled("Total Requests: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", metrics.total_requests),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Errors: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", metrics.total_errors),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Error Rate: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.2}%", metrics.error_rate),
                    Style::default()
                        .fg(if metrics.error_rate > 5.0 {
                            Color::Red
                        } else {
                            Color::Green
                        })
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Routes: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.routes.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("API Key Pools: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", self.config.api_key_pools.len()),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ];

        let metrics_widget = Paragraph::new(metrics_text)
            .block(Block::default().borders(Borders::ALL).title("📊 Metrics"))
            .wrap(Wrap { trim: true });
        f.render_widget(metrics_widget, chunks[0]);

        // Right side: Health
        let health_status_color = match health_response.status {
            crate::health::HealthStatus::Healthy => Color::Green,
            crate::health::HealthStatus::Unhealthy => Color::Red,
            crate::health::HealthStatus::Degraded => Color::Yellow,
        };

        let mut health_text = vec![
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}", health_response.status),
                    Style::default()
                        .fg(health_status_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    health_response.version.clone(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Uptime: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.health.uptime_formatted(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(""),
        ];

        // Display all servers
        let servers = self.config.get_servers();
        health_text.push(Line::from(vec![
            Span::styled("Servers: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", servers.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]));
        for server in servers {
            let name = server
                .name
                .clone()
                .unwrap_or_else(|| format!("{}:{}", server.host, server.port));
            health_text.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(name, Style::default().fg(Color::White)),
            ]));
        }

        health_text.push(Line::from(""));
        health_text.push(Line::from(vec![
            Span::styled("Metrics: ", Style::default().fg(Color::Gray)),
            Span::styled(
                if self.config.metrics.enabled {
                    self.config.metrics.path.clone()
                } else {
                    "disabled".to_string()
                },
                Style::default().fg(Color::White),
            ),
        ]));

        let health_widget = Paragraph::new(health_text)
            .block(Block::default().borders(Borders::ALL).title("💚 Health"))
            .wrap(Wrap { trim: true });
        f.render_widget(health_widget, chunks[1]);
    }

    fn render_routes(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Left: Route list
        let items: Vec<ListItem> = self
            .routes
            .iter()
            .map(|route| {
                let content = format!("{} → {}", route.path_pattern, route.target);
                ListItem::new(content).style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Routes"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, chunks[0], &mut self.route_list_state);

        // Right: Route details
        let detail_text = if let Some(selected) = self.route_list_state.selected() {
            if selected < self.routes.len() {
                let route = &self.routes[selected];
                let methods = if route.methods.is_empty() {
                    "ALL".to_string()
                } else {
                    route.methods.join(", ")
                };
                let api_key = route
                    .api_key_selector
                    .as_ref()
                    .map(|s| format!("{} ({})", s.header_name, s.strategy_name()))
                    .unwrap_or_else(|| "None".to_string());

                vec![
                    Line::from(vec![
                        Span::styled("Path: ", Style::default().fg(Color::Gray)),
                        Span::styled(route.path_pattern.clone(), Style::default().fg(Color::Cyan)),
                    ]),
                    Line::from(vec![
                        Span::styled("Target: ", Style::default().fg(Color::Gray)),
                        Span::styled(route.target.clone(), Style::default().fg(Color::Green)),
                    ]),
                    Line::from(vec![
                        Span::styled("Methods: ", Style::default().fg(Color::Gray)),
                        Span::styled(methods, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(vec![
                        Span::styled("Strip Prefix: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            if route.strip_prefix { "Yes" } else { "No" },
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("API Key: ", Style::default().fg(Color::Gray)),
                        Span::styled(api_key, Style::default().fg(Color::Magenta)),
                    ]),
                    Line::from(""),
                    Line::from(vec![Span::styled(
                        route
                            .description
                            .clone()
                            .unwrap_or_else(|| "No description".to_string()),
                        Style::default().fg(Color::DarkGray),
                    )]),
                ]
            } else {
                vec![Line::from("Select a route")]
            }
        } else {
            vec![Line::from("No routes configured")]
        };

        let detail = Paragraph::new(detail_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Route Details"),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(detail, chunks[1]);
    }

    fn render_config(&self, f: &mut Frame, area: Rect) {
        let mut config_text = vec![Line::from(Span::styled(
            "Servers Configuration",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))];

        // Display all servers
        let servers = self.config.get_servers();
        for server in servers {
            let name = server
                .name
                .clone()
                .unwrap_or_else(|| format!("{}:{}", server.host, server.port));
            let route_count = self.config.routes_for_server(server).len();
            config_text.push(Line::from(format!("  {}:", name)));
            config_text.push(Line::from(format!(
                "    Address: {}:{}",
                server.host, server.port
            )));
            config_text.push(Line::from(format!("    Timeout: {}s", server.timeout)));
            config_text.push(Line::from(format!("    Routes: {}", route_count)));
        }

        config_text.push(Line::from(""));
        config_text.push(Line::from(Span::styled(
            "Metrics Configuration",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        config_text.push(Line::from(format!(
            "  Enabled: {}",
            if self.config.metrics.enabled {
                "Yes"
            } else {
                "No"
            }
        )));
        config_text.push(Line::from(format!("  Path: {}", self.config.metrics.path)));
        config_text.push(Line::from(""));
        config_text.push(Line::from(Span::styled(
            "Health Configuration",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));
        config_text.push(Line::from(format!(
            "  Enabled: {}",
            if self.config.health.enabled {
                "Yes"
            } else {
                "No"
            }
        )));
        config_text.push(Line::from(format!("  Path: {}", self.config.health.path)));
        config_text.push(Line::from(""));
        config_text.push(Line::from(Span::styled(
            "API Key Pools",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )));

        for (name, pool) in &self.config.api_key_pools {
            config_text.push(Line::from(format!("  {}:", name)));
            config_text.push(Line::from(format!("    Strategy: {:?}", pool.strategy)));
            config_text.push(Line::from(format!("    Header: {}", pool.header_name)));
            config_text.push(Line::from(format!(
                "    Keys: {} ({} enabled)",
                pool.keys.len(),
                pool.keys.iter().filter(|k| k.enabled).count()
            )));
        }

        let config = Paragraph::new(config_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("⚙️ Configuration"),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(config, area);
    }

    fn render_help(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  Tab / →         Next tab"),
            Line::from("  Shift+Tab / ←   Previous tab"),
            Line::from("  1-4             Jump to tab"),
            Line::from("  h               Help tab"),
            Line::from("  q / Esc         Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "Routes Tab",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("  ↑ / k           Previous route"),
            Line::from("  ↓ / j           Next route"),
            Line::from(""),
            Line::from(Span::styled(
                "About",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("  Open Gateway v{}", env!("CARGO_PKG_VERSION"))),
            Line::from("  A simple and fast API gateway service"),
            Line::from(""),
            Line::from("  https://github.com/npv2k1/open-gateway"),
        ];

        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title("❓ Help"))
            .wrap(Wrap { trim: true });
        f.render_widget(help, area);
    }

    fn render_status_bar(&self, f: &mut Frame, area: Rect) {
        let status = Paragraph::new(Line::from(vec![
            Span::styled("Tab/←→", Style::default().fg(Color::Yellow)),
            Span::raw(": Switch tabs  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(": Quit  "),
            Span::styled("h", Style::default().fg(Color::Yellow)),
            Span::raw(": Help"),
        ]))
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, area);
    }
}
