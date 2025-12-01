mod detail;
mod dialogs;
mod layout;
mod list;

use ratatui::Frame;

use crate::app::{App, AppMode};

pub fn render(frame: &mut Frame, app: &App) {
    let areas = layout::calculate_layout(frame);

    // Header
    layout::render_header(frame, areas.header);

    // Worktree list
    list::render(frame, app, areas.list);

    // Detail panel
    detail::render(frame, app, areas.detail);

    // Footer with keybindings
    layout::render_footer(frame, app, areas.footer);

    // Modal overlays
    match app.mode {
        AppMode::Creating => {
            dialogs::render_create_dialog(frame, app);
        }
        AppMode::ConfirmDelete => {
            dialogs::render_delete_dialog(frame, app);
        }
        AppMode::Help => {
            dialogs::render_help(frame);
        }
        AppMode::Normal => {}
    }
}
