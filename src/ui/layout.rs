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
    // Show error if present
    if let Some(error) = &app.error {
        let error_line = Line::from(vec![
            Span::styled(" Error: ", Style::default().fg(Color::Red)),
            Span::raw(error.as_str()),
        ]);
        let keybindings = Line::from(vec![Span::styled(
            " n:new d:del e:edit g:git c:ide m:merge t:toggle r:refresh ?:help q:quit ",
            Style::default().fg(Color::DarkGray),
        )]);
        let footer = Paragraph::new(vec![error_line, keybindings]);
        frame.render_widget(footer, area);
    } else {
        let keybindings = Line::from(vec![Span::styled(
            " n:new d:del e:edit g:git c:ide m:merge t:toggle Enter:cd r:refresh ?:help q:quit ",
            Style::default().fg(Color::DarkGray),
        )]);
        let footer = Paragraph::new(vec![Line::default(), keybindings]);
        frame.render_widget(footer, area);
    }
}
