use std::collections::HashSet;
use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;

use crate::model::{Event, EventKind, MonitorRequest};
use crate::monitor::MonitorService;

pub async fn run_dashboard(service: MonitorService, handle: &str, tick_ms: u64) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = dashboard_loop(&mut terminal, service, handle, tick_ms).await;

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

async fn dashboard_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    service: MonitorService,
    handle: &str,
    tick_ms: u64,
) -> Result<()> {
    let request = MonitorRequest {
        handle: handle.to_string(),
        kinds: vec![EventKind::Tweet, EventKind::Reply],
        limit: 5,
    };
    let tick_rate = Duration::from_millis(tick_ms);
    let mut last_tick = Instant::now();
    let mut events: Vec<Event> = Vec::new();
    let mut seen_ids = HashSet::new();
    let mut last_error: Option<String> = None;

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(8)])
                .split(frame.size());

            let status_line = match &last_error {
                Some(error) => format!(
                    "Xenon Dashboard  profile={}  events={}  error={}  press q to quit",
                    handle,
                    events.len(),
                    error
                ),
                None => format!(
                    "Xenon Dashboard  profile={}  events={}  press q to quit",
                    handle,
                    events.len()
                ),
            };

            let header = Paragraph::new(status_line)
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );

            let items = events
                .iter()
                .rev()
                .take(12)
                .map(|event| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("[{}] ", event.kind),
                            Style::default().fg(Color::Yellow),
                        ),
                        Span::raw(format!("{} | score {}", event.message, event.score)),
                    ]))
                })
                .collect::<Vec<_>>();

            let list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Live Feed"));

            frame.render_widget(header, chunks[0]);
            frame.render_widget(list, chunks[1]);
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if matches!(key.code, KeyCode::Char('q')) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            match service.generate_batch(&request).await {
                Ok(batch) => {
                    last_error = None;
                    for event in batch {
                        if seen_ids.insert(event.id.clone()) {
                            events.push(event);
                        }
                    }
                }
                Err(error) => {
                    last_error = Some(error.to_string());
                }
            }
            last_tick = Instant::now();
        }
    }

    Ok(())
}
