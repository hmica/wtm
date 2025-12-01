mod app;
mod git;
mod status;
mod ui;

use anyhow::Result;
use app::App;
use std::env;
use std::io::{self, Write};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle -m flag: go directly to main worktree
    if args.iter().any(|a| a == "-m" || a == "--main") {
        let repo_path = std::env::current_dir()?;
        let worktrees = git::list_worktrees(&repo_path)?;
        if let Some(main_wt) = worktrees.into_iter().find(|w| w.is_main) {
            writeln!(io::stderr(), "{}", main_wt.path.display())?;
        }
        return Ok(());
    }

    // Setup panic handler to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = ratatui::restore();
        original_hook(panic);
    }));

    // Initialize terminal
    let terminal = ratatui::init();

    // Run app
    let mut app = App::new()?;
    let result = app.run(terminal);

    // Cleanup
    ratatui::restore();

    // If user selected a worktree to exit into, print path to stderr
    // (stdout is used by TUI, stderr is captured by shell wrapper)
    if let Some(path) = &app.exit_path {
        writeln!(io::stderr(), "{}", path.display())?;
    }

    result
}
