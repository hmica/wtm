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
    let area = centered_rect(55, 40, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Delete Worktree ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(wt) = app.selected_worktree() {
        let branch = wt.branch.as_deref().unwrap_or("(detached)");
        let has_unmerged = wt.ahead > 0;
        let is_dangerous = wt.has_changes || has_unmerged;

        // Build warning lines
        let mut lines: Vec<Line> = vec![];

        if is_dangerous {
            lines.push(Line::from(Span::styled(
                " ⚠ WARNING - DATA LOSS RISK ⚠",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            if wt.has_changes {
                lines.push(Line::from(vec![
                    Span::styled(" • ", Style::default().fg(Color::Red)),
                    Span::styled("Uncommitted changes", Style::default().fg(Color::Red)),
                    Span::raw(" (will force delete)"),
                ]));
            }

            if has_unmerged {
                lines.push(Line::from(vec![
                    Span::styled(" • ", Style::default().fg(Color::Red)),
                    Span::styled(
                        format!("{} unmerged commit(s)", wt.ahead),
                        Style::default().fg(Color::Red),
                    ),
                    Span::raw(" (will be lost!)"),
                ]));
            }

            lines.push(Line::from(""));
        }

        lines.push(Line::from(format!(" Branch: {}", branch)));
        lines.push(Line::from(format!(" Path: {}", wt.path.display())));
        lines.push(Line::from(""));

        if is_dangerous {
            lines.push(Line::from(vec![
                Span::styled(" Force delete? ", Style::default().fg(Color::Red)),
                Span::styled("y", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("/"),
                Span::styled("n", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw(" Delete this worktree? "),
                Span::styled("y", Style::default().fg(Color::Green)),
                Span::raw("/"),
                Span::styled("n", Style::default().fg(Color::Red)),
            ]));
        }

        let content = Paragraph::new(lines);
        frame.render_widget(content, inner);
    }
}

pub fn render_deleting(frame: &mut Frame, app: &App) {
    let area = centered_rect(40, 20, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Deleting ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if let Some(wt) = app.selected_worktree() {
        let branch = wt.branch.as_deref().unwrap_or("(detached)");
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Deleting worktree...",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(format!("  {}", branch)),
        ]);
        frame.render_widget(content, inner);
    }
}

pub fn render_help(frame: &mut Frame, app: &App) {
    use crate::config::Shortcut;

    let area = centered_rect(55, 70, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ")
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Navigation (hardcoded)",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/k, ↑/↓    Move selection"),
        Line::from("  Tab         Toggle notes/git status view"),
        Line::from(""),
        Line::from(Span::styled(
            " Shortcuts (from config)",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];

    // Get shortcuts from config and sort by key
    let mut shortcuts: Vec<_> = app.config.shortcuts.iter().collect();
    shortcuts.sort_by(|a, b| a.0.cmp(b.0));

    for (key, shortcut) in shortcuts {
        let description = match shortcut {
            Shortcut::BuiltIn { action } => {
                match action.as_str() {
                    "create" => "Create new worktree".to_string(),
                    "delete" => "Delete worktree".to_string(),
                    "edit" => "Edit status file".to_string(),
                    "merge_main" => "Merge main (ff-only)".to_string(),
                    "toggle_view" => "Toggle notes/git view".to_string(),
                    "refresh" => "Refresh list".to_string(),
                    "help" => "Toggle this help".to_string(),
                    "quit" => "Quit".to_string(),
                    "cd" => "Exit and cd to worktree".to_string(),
                    _ => format!("Action: {}", action),
                }
            }
            Shortcut::Command { cmd, mode } => {
                let mode_str = match mode {
                    crate::config::CommandMode::Replace => "replace",
                    crate::config::CommandMode::Detach => "detach",
                };
                // Truncate long commands
                let cmd_short = if cmd.len() > 25 {
                    format!("{}...", &cmd[..22])
                } else {
                    cmd.clone()
                };
                format!("{} ({})", cmd_short, mode_str)
            }
        };

        let key_display = if key == "Enter" { "Enter" } else { key };
        lines.push(Line::from(format!("  {:<10}  {}", key_display, description)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Config: ~/.config/wtm/config.toml",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        " Press any key to close",
        Style::default().fg(Color::DarkGray),
    )));

    let help = Paragraph::new(lines);
    frame.render_widget(help, inner);
}
