use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub struct AppLayout {
    pub header: Rect,
    pub list: Rect,
    pub detail: Rect,
    pub footer: Rect,
}

pub fn calculate_layout(frame: &Frame) -> AppLayout {
    let area = frame.area();

    // Vertical: header (1) | main (fill) | footer (2)
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Horizontal: list (40%) | detail (60%)
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(vertical[1]);

    AppLayout {
        header: vertical[0],
        list: horizontal[0],
        detail: horizontal[1],
        footer: vertical[2],
    }
}

pub fn render_header(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" wtm ", Style::default().fg(Color::Cyan)),
        Span::raw("- Git Worktree Manager"),
    ]));
    frame.render_widget(title, area);
}

pub fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keybindings = Line::from(vec![Span::styled(
        " n:new d:del e:edit g:git c:ide m:merge t:toggle Enter:cd l:logs r:refresh ?:help q:quit ",
        Style::default().fg(Color::DarkGray),
    )]);

    // Line 1 priority: error > running init job > finished init job > blank.
    let status_line: Line = if let Some(error) = &app.error {
        Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(Color::Red)),
            Span::raw(error.as_str()),
        ])
    } else if let Some(job) = app.init_jobs.first() {
        const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let elapsed = job.started.elapsed();
        let frame_idx = (elapsed.as_millis() / 120) as usize % FRAMES.len();
        let extra = if app.init_jobs.len() > 1 {
            format!(" (+{} more)", app.init_jobs.len() - 1)
        } else {
            String::new()
        };
        Line::from(vec![Span::styled(
            format!(
                " {} init running for {} ({}s){} ",
                FRAMES[frame_idx],
                job.branch,
                elapsed.as_secs(),
                extra,
            ),
            Style::default().fg(Color::Yellow),
        )])
    } else if let Some(outcome) = &app.init_outcome {
        if outcome.success {
            Line::from(vec![Span::styled(
                format!(" init ✓ {} ", outcome.branch),
                Style::default().fg(Color::Green),
            )])
        } else {
            Line::from(vec![Span::styled(
                format!(" init ✗ {} (failed — press l for logs) ", outcome.branch),
                Style::default().fg(Color::Red),
            )])
        }
    } else {
        Line::default()
    };

    let footer = Paragraph::new(vec![status_line, keybindings]);
    frame.render_widget(footer, area);
}
