use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_create_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Create Worktree ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Label
            Constraint::Length(3), // Input
            Constraint::Length(1), // Suggestions label
            Constraint::Min(0),    // Suggestions list
            Constraint::Length(1), // Help
        ])
        .margin(1)
        .split(inner);

    // Branch name label
    let label = Paragraph::new("Branch name (new or existing):");
    frame.render_widget(label, chunks[0]);

    // Input field with cursor
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let input_text = if app.input.is_empty() {
        Span::styled("type to search...", Style::default().fg(Color::DarkGray))
    } else {
        Span::raw(&app.input)
    };

    let input = Paragraph::new(input_text).block(input_block);
    frame.render_widget(input, chunks[1]);

    // Show cursor position
    let cursor_x = chunks[1].x + 1 + app.input_cursor as u16;
    let cursor_y = chunks[1].y + 1;
    frame.set_cursor_position((cursor_x, cursor_y));

    // Suggestions label
    if !app.filtered_branches.is_empty() {
        let suggestions_label = Paragraph::new(Span::styled(
            format!("Matching branches ({}):", app.filtered_branches.len()),
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(suggestions_label, chunks[2]);

        // Suggestions list
        let items: Vec<ListItem> = app
            .filtered_branches
            .iter()
            .take(8)
            .map(|b| {
                ListItem::new(Line::from(Span::styled(
                    format!("  {}", b),
                    Style::default().fg(Color::Yellow),
                )))
            })
            .collect();

        let suggestions = List::new(items);
        frame.render_widget(suggestions, chunks[3]);
    } else if !app.input.is_empty() {
        let new_branch_hint = Paragraph::new(Span::styled(
            "Will create new branch",
            Style::default().fg(Color::Green),
        ));
        frame.render_widget(new_branch_hint, chunks[2]);
    }

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": create  "),
        Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": autocomplete  "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": cancel"),
    ]));
    frame.render_widget(help, chunks[4]);
}

pub fn render_delete_dialog(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delete Worktree ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(wt) = app.selected_worktree() {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .margin(1)
            .split(inner);

        let branch = wt.branch.as_deref().unwrap_or("(detached)");

        let warning = if wt.has_changes {
            Paragraph::new(Line::from(vec![
                Span::styled("WARNING: ", Style::default().fg(Color::Red)),
                Span::raw("Worktree has uncommitted changes!"),
            ]))
        } else {
            Paragraph::new("")
        };
        frame.render_widget(warning, chunks[0]);

        let info = Paragraph::new(vec![
            Line::from(format!("Branch: {}", branch)),
            Line::from(format!("Path: {}", wt.path.display())),
        ]);
        frame.render_widget(info, chunks[1]);

        let confirm = Paragraph::new(Line::from(vec![
            Span::raw("Delete this worktree? "),
            Span::styled("y", Style::default().fg(Color::Green)),
            Span::raw("/"),
            Span::styled("n", Style::default().fg(Color::Red)),
        ]));
        frame.render_widget(confirm, chunks[2]);
    }
}

pub fn render_help(frame: &mut Frame) {
    let area = centered_rect(50, 60, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Navigation",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k, ↑/↓    Move selection"),
        Line::from("  Enter       Exit and cd to worktree"),
        Line::from("  t/Tab       Toggle notes/git status view"),
        Line::from(""),
        Line::from(Span::styled(
            " Actions",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  n           Create new worktree"),
        Line::from("  d           Delete worktree"),
        Line::from("  e           Edit status file in $EDITOR"),
        Line::from("  g           Open lazygit"),
        Line::from("  c           Open in IDE ($CODE_IDE)"),
        Line::from("  m           Merge main (ff-only)"),
        Line::from("  r           Refresh list"),
        Line::from(""),
        Line::from(Span::styled(
            " Other",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?           Toggle this help"),
        Line::from("  q           Quit"),
        Line::from(""),
        Line::from(Span::styled(
            " Press any key to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let help = Paragraph::new(help_text);
    frame.render_widget(help, inner);
}
