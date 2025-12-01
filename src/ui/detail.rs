use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, DetailViewMode};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let (title, content) = if let Some(wt) = app.selected_worktree() {
        match app.detail_view {
            DetailViewMode::Notes => {
                let title = " Notes [t:git] ";
                let lines = if let Some(status_content) = &app.status_content {
                    // Render status file with basic syntax highlighting
                    status_content
                        .lines()
                        .map(|line| {
                            if line.starts_with("# ") {
                                Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
                            } else if line.starts_with("## ") {
                                Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
                            } else if line.starts_with("- [x]") || line.starts_with("- [X]") {
                                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
                            } else if line.starts_with("- [ ]") {
                                Line::from(Span::styled(line, Style::default().fg(Color::Red)))
                            } else if line.starts_with("<!--") || line.ends_with("-->") {
                                Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)))
                            } else {
                                Line::from(line)
                            }
                        })
                        .collect()
                } else {
                    // No status file
                    vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "  No .worktree-status.md file",
                            Style::default().fg(Color::DarkGray),
                        )),
                        Line::from(""),
                        Line::from(Span::styled(
                            "  Press 'e' to create one",
                            Style::default().fg(Color::DarkGray),
                        )),
                        Line::from(""),
                        Line::from(format!("  Path: {}", wt.path.display())),
                        Line::from(format!(
                            "  Branch: {}",
                            wt.branch.as_deref().unwrap_or("(detached)")
                        )),
                        Line::from(format!("  Commit: {}", wt.commit)),
                    ]
                };
                (title, lines)
            }
            DetailViewMode::GitStatus => {
                let title = " Git Status [t:notes] ";
                let lines = if let Some(status_content) = &app.status_content {
                    status_content
                        .lines()
                        .map(|line| {
                            if line.starts_with("M ") || line.starts_with(" M") {
                                // Modified
                                Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
                            } else if line.starts_with("A ") || line.starts_with("?? ") {
                                // Added / Untracked
                                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
                            } else if line.starts_with("D ") || line.starts_with(" D") {
                                // Deleted
                                Line::from(Span::styled(line, Style::default().fg(Color::Red)))
                            } else if line.starts_with("R ") {
                                // Renamed
                                Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
                            } else if line == "Working tree clean" {
                                Line::from(Span::styled(
                                    format!("  {}", line),
                                    Style::default().fg(Color::Green),
                                ))
                            } else {
                                Line::from(format!(" {}", line))
                            }
                        })
                        .collect()
                } else {
                    vec![Line::from(Span::styled(
                        "  Unable to get git status",
                        Style::default().fg(Color::Red),
                    ))]
                };
                (title, lines)
            }
        }
    } else {
        (
            " Status ",
            vec![Line::from(Span::styled(
                "  No worktrees found",
                Style::default().fg(Color::DarkGray),
            ))],
        )
    };

    let paragraph = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
