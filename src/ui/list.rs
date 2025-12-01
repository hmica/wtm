use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .worktrees
        .iter()
        .map(|wt| {
            let (checked, total) = wt.status.progress;
            let progress = if total > 0 {
                format!("[{}/{}]", checked, total)
            } else {
                "[---]".to_string()
            };

            let branch_name = wt.branch.as_deref().unwrap_or("(detached)");
            let main_marker = if wt.is_main { "(main)" } else { "" };

            // Determine if branch is merged and ready to delete (ahead=0, clean, not main)
            let is_merged = !wt.is_main && wt.ahead == 0 && !wt.has_changes;

            // Indicator: * for dirty, ✓ for merged, space otherwise
            let indicator = if wt.has_changes {
                "*"
            } else if is_merged {
                "✓"
            } else {
                " "
            };

            let display_name = if wt.is_main {
                format!("{} {}", branch_name, main_marker)
            } else {
                branch_name.to_string()
            };

            // Build ahead/behind indicator for non-main branches
            let ahead_behind = if !wt.is_main && (wt.ahead > 0 || wt.behind > 0) {
                format!(" ↑{}↓{}", wt.ahead, wt.behind)
            } else {
                String::new()
            };

            // Color: green for main or merged branches, cyan for others
            let branch_color = if wt.is_main || is_merged {
                Color::Green
            } else {
                Color::Cyan
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{} ", indicator),
                    if wt.has_changes {
                        Style::default().fg(Color::Yellow)
                    } else if is_merged {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(
                    format!("{:<24}", display_name),
                    Style::default().fg(branch_color),
                ),
                Span::styled(
                    ahead_behind,
                    Style::default().fg(Color::Magenta),
                ),
                Span::styled(format!(" {}", progress), Style::default().fg(Color::Yellow)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Worktrees "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.list_state.clone());
}
