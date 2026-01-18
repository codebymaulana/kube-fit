use std::{io, time::Duration};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use crate::models::{MemoryRequest, MemoryUsage};
use std::collections::HashMap;


#[derive(Clone, Copy, PartialEq)]
enum FilterStatus {
    All,
    Overutilized,
    Underutilized,
    Normal,
}

impl FilterStatus {
    fn next(&self) -> Self {
        match self {
            FilterStatus::All => FilterStatus::Overutilized,
            FilterStatus::Overutilized => FilterStatus::Underutilized,
            FilterStatus::Underutilized => FilterStatus::Normal,
            FilterStatus::Normal => FilterStatus::All,
        }
    }

    fn to_string(&self) -> String {
        match self {
            FilterStatus::All => "ALL".to_string(),
            FilterStatus::Overutilized => "OVERUTILIZED".to_string(),
            FilterStatus::Underutilized => "UNDERUTILIZED".to_string(),
            FilterStatus::Normal => "NORMAL".to_string(),
        }
    }
}

enum InputMode {
    Normal,
    Editing,
}

#[derive(Clone)]
struct PodRow {
    namespace: String,
    name: String,
    status: String,
    usage: f64,
}

struct AppState {
    all_items: Vec<PodRow>,
    visible_items: Vec<PodRow>,
    table_state: TableState,
    status_filter: FilterStatus,
    search_query: String,
    input_mode: InputMode,
}

impl AppState {
    fn new(items: Vec<PodRow>) -> Self {
        let mut app = Self {
            all_items: items.clone(),
            visible_items: items,
            table_state: TableState::default(),
            status_filter: FilterStatus::All,
            search_query: String::new(),
            input_mode: InputMode::Normal,
        };
        app.table_state.select(Some(0));
        app
    }

    fn apply_filters(&mut self) {
        self.visible_items = self.all_items
            .iter()
            .filter(|item| {
                // Status Filter
                let status_match = match self.status_filter {
                    FilterStatus::All => true,
                    FilterStatus::Overutilized => item.status == "Overutilized",
                    FilterStatus::Underutilized => item.status == "Underutilized",
                    FilterStatus::Normal => item.status == "Normal",
                };

                // Text Search (Namespace, Name, Status)
                let query = self.search_query.to_lowercase();
                let text_match = if self.search_query.is_empty() {
                    true
                } else {
                    item.name.to_lowercase().contains(&query) || 
                    item.namespace.to_lowercase().contains(&query) || 
                    item.status.to_lowercase().contains(&query)
                };

                status_match && text_match
            })
            .cloned()
            .collect();

        // Reset
        if self.visible_items.is_empty() {
            self.table_state.select(None);
        } else {
            self.table_state.select(Some(0));
        }
    }
}

// TUI runner

pub fn run_tui(requests: Vec<MemoryRequest>, usages: Vec<MemoryUsage>) -> Result<(), io::Error> {
    
    let mut rows_processed = Vec::new();
    let mut request_map: HashMap<String, Vec<crate::models::MetricPoint>> = HashMap::new();
    
    for req in requests { 
        request_map.insert(req.pod_name, req.metrics); 
    }

    for usage in usages {
        if let Some(req_metrics) = request_map.get(&usage.pod_name) {
            let mut total_percentage = 0.0;
            let mut count = 0.0;
            for (u_point, r_point) in usage.metrics.iter().zip(req_metrics.iter()) {
                if r_point.value > 0.0 {
                    total_percentage += (u_point.value / r_point.value) * 100.0;
                    count += 1.0;
                }
            }
            if count > 0.0 {
                let avg = total_percentage / count;
                let status_raw = if avg >= 90.0 { "Overutilized" } 
                                 else if avg <= 10.0 { "Underutilized" } 
                                 else { "Normal" };
                
                rows_processed.push(PodRow {
                    namespace: usage.namespace,
                    name: usage.pod_name,
                    status: status_raw.to_string(),
                    usage: avg,
                });
            }
        }
    }

    // TUI setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new(rows_processed);
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if crossterm::event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => next(app),
                        KeyCode::Up => previous(app),
                        KeyCode::Tab => {
                            app.status_filter = app.status_filter.next();
                            app.apply_filters();
                        },
                        KeyCode::Char('/') => app.input_mode = InputMode::Editing,
                        KeyCode::Esc => {
                            app.search_query.clear();
                            app.status_filter = FilterStatus::All;
                            app.apply_filters();
                        },
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter | KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.apply_filters();
                        },
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.apply_filters();
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

// UI rendering

fn ui(f: &mut Frame, app: &mut AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
        .split(f.area());

    // Header (Cyan BG, Black Text)
    let header_style = Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD);
    let header = Row::new(vec!["NAMESPACE", "POD NAME", "STATUS", "AVG USAGE"])
        .style(header_style)
        .height(1)
        .bottom_margin(1);

    let selected_index = app.table_state.selected();

    let rows = app.visible_items.iter().enumerate().map(|(i, item)| {
        let is_selected = Some(i) == selected_index;

        let status_color = match item.status.as_str() {
            "Overutilized" => Color::Red,
            "Underutilized" => Color::Yellow,
            _ => Color::Green,
        };

        // Selection Logic (Light Gray Selection)
        let (base_style, status_style) = if is_selected {
            (
                Style::default().bg(Color::Gray).fg(Color::Black),
                Style::default().bg(Color::Gray).fg(Color::Black),
            )
        } else {
            (
                Style::default(),
                Style::default().fg(status_color),
            )
        };

        let cells = vec![
            Cell::from(item.namespace.clone()).style(base_style),
            Cell::from(item.name.clone()).style(base_style),
            Cell::from(item.status.clone()).style(status_style),
            Cell::from(format!("{:.2}%", item.usage)).style(status_style),
        ];

        Row::new(cells).height(1).bottom_margin(0)
    });

    let title = format!(" Pod Usage Analyzer [{}] (Show: {}) ", app.visible_items.len(), app.status_filter.to_string());

    let t = Table::new(
        rows,
        [
            Constraint::Percentage(20), // Namespace
            Constraint::Percentage(40), // Pod
            Constraint::Percentage(25), // Status
            Constraint::Percentage(15)  // Usage
        ]
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(title))
    .column_spacing(0) // No black gaps
    .highlight_style(Style::default()) 
    .highlight_symbol(">> ");

    f.render_stateful_widget(t, chunks[0], &mut app.table_state);

    // Search Bar
    let search_text = format!("Search: {}", app.search_query);
    let search_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::Gray),
        InputMode::Editing => Style::default().fg(Color::Yellow),
    };
    let p = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).title(" Controls: [Tab] Filter Status | [/] Search | [Esc] Reset | [q] Quit ").style(search_style));
    f.render_widget(p, chunks[1]);
}

fn next(app: &mut AppState) {
    if app.visible_items.is_empty() { return; }
    let i = match app.table_state.selected() {
        Some(i) => if i >= app.visible_items.len() - 1 { 0 } else { i + 1 },
        None => 0,
    };
    app.table_state.select(Some(i));
}

fn previous(app: &mut AppState) {
    if app.visible_items.is_empty() { return; }
    let i = match app.table_state.selected() {
        Some(i) => if i == 0 { app.visible_items.len() - 1 } else { i - 1 },
        None => 0,
    };
    app.table_state.select(Some(i));
}