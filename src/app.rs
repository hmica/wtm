use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui::DefaultTerminal;

use crate::config::{CommandMode, Config, Shortcut};
use crate::git::Worktree;
use crate::ui;

#[derive(Default, PartialEq)]
pub enum AppMode {
    #[default]
    Normal,
    Creating,
    ConfirmDelete,
    Deleting,
    Help,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum DetailViewMode {
    #[default]
    Notes,
    GitStatus,
}

pub struct App {
    pub worktrees: Vec<Worktree>,
    pub selected: usize,
    pub list_state: ListState,
    pub mode: AppMode,
    pub detail_view: DetailViewMode,
    pub status_content: Option<String>,
    pub input: String,
    pub input_cursor: usize,
    pub should_quit: bool,
    pub error: Option<String>,
    pub repo_path: PathBuf,
    pub branches: Vec<String>,
    pub filtered_branches: Vec<String>,
    pub exit_path: Option<PathBuf>,
    pub needs_full_redraw: bool,
    pub config: Config,
}

impl App {
    pub fn new() -> Result<Self> {
        let repo_path = std::env::current_dir()?;
        let config = match Config::load() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Could not load config: {}. Using defaults.", e);
                Config::default()
            }
        };
        let mut app = Self {
            worktrees: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
            mode: AppMode::Normal,
            detail_view: DetailViewMode::Notes,
            status_content: None,
            input: String::new(),
            input_cursor: 0,
            should_quit: false,
            error: None,
            repo_path,
            branches: Vec::new(),
            filtered_branches: Vec::new(),
            exit_path: None,
            needs_full_redraw: false,
            config,
        };
        app.list_state.select(Some(0));
        Ok(app)
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        // Clear screen on startup to remove any previous terminal content
        terminal.clear()?;

        // Initial load
        self.refresh_worktrees();
        self.refresh_branches();

        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        while !self.should_quit {
            // Force full redraw if needed (e.g., after returning from editor)
            if self.needs_full_redraw {
                terminal.clear()?;
                self.needs_full_redraw = false;
            }

            // Render
            terminal.draw(|frame| ui::render(frame, self))?;

            // Perform delete after showing "Deleting..." UI
            if self.mode == AppMode::Deleting {
                self.delete_worktree()?;
                continue;
            }

            // Poll for events with timeout
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                self.handle_event(event::read()?)?;
            }

            // Tick
            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                // Clear error on any keypress
                self.error = None;
                self.handle_key(key.code)?;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match self.mode {
            AppMode::Normal => self.handle_normal_key(key),
            AppMode::Creating => self.handle_creating_key(key),
            AppMode::ConfirmDelete => self.handle_delete_key(key),
            AppMode::Deleting => Ok(()), // Ignore input while deleting
            AppMode::Help => self.handle_help_key(key),
        }
    }

    fn handle_normal_key(&mut self, key: KeyCode) -> Result<()> {
        // Navigation keys are always hardcoded
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                return Ok(());
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_previous();
                return Ok(());
            }
            KeyCode::Tab => {
                self.toggle_detail_view();
                return Ok(());
            }
            _ => {}
        }

        // Convert key to config key string
        let key_str = match key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            _ => return Ok(()),
        };

        // Look up shortcut in config
        if let Some(shortcut) = self.config.get_shortcut(&key_str).cloned() {
            match shortcut {
                Shortcut::BuiltIn { action } => self.run_builtin_action(&action)?,
                Shortcut::Command { cmd, mode } => self.run_command(&cmd, mode)?,
            }
        }

        Ok(())
    }

    fn run_builtin_action(&mut self, action: &str) -> Result<()> {
        match action {
            "quit" => self.should_quit = true,
            "create" => self.start_create(),
            "delete" => self.start_delete(),
            "edit" => self.open_editor()?,
            "merge_main" => self.merge_main()?,
            "toggle_view" => self.toggle_detail_view(),
            "refresh" => {
                self.refresh_worktrees();
                self.refresh_branches();
            }
            "help" => self.mode = AppMode::Help,
            "cd" => self.exit_to_worktree(),
            _ => {
                self.error = Some(format!("Unknown action: {}", action));
            }
        }
        Ok(())
    }

    fn run_command(&mut self, cmd: &str, mode: CommandMode) -> Result<()> {
        let Some(wt) = self.worktrees.get(self.selected) else {
            return Ok(());
        };

        let branch = wt.branch.as_deref().unwrap_or("detached");
        let path = wt.path.to_string_lossy();
        let repo_path = self.repo_path.to_string_lossy();

        // Expand variables in command
        let expanded_cmd = cmd
            .replace("$1", &path)
            .replace("$path", &path)
            .replace("$2", branch)
            .replace("$branch", branch)
            .replace("$repo", &repo_path);

        match mode {
            CommandMode::Replace => {
                // Take over terminal (like lazygit)
                ratatui::restore();

                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&expanded_cmd)
                    .current_dir(&wt.path)
                    .status();

                let _ = ratatui::init();

                if let Err(e) = status {
                    self.error = Some(format!("Command failed: {}", e));
                }

                self.refresh_worktrees();
                self.refresh_branches();
                self.load_status_content();
                self.needs_full_redraw = true;
            }
            CommandMode::Detach => {
                // Spawn in background (like IDE)
                let result = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&expanded_cmd)
                    .spawn();

                if let Err(e) = result {
                    self.error = Some(format!("Failed to spawn: {}", e));
                }
            }
        }

        Ok(())
    }

    fn handle_creating_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.input.clear();
                self.input_cursor = 0;
                self.filtered_branches.clear();
            }
            KeyCode::Enter => {
                if !self.input.is_empty() {
                    self.create_worktree()?;
                }
            }
            KeyCode::Backspace => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input.remove(self.input_cursor);
                    self.update_filtered_branches();
                }
            }
            KeyCode::Left => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
            }
            KeyCode::Tab => {
                // Autocomplete from filtered branches
                if let Some(branch) = self.filtered_branches.first() {
                    self.input = branch.clone();
                    self.input_cursor = self.input.len();
                    self.update_filtered_branches();
                }
            }
            KeyCode::Char(c) => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += 1;
                self.update_filtered_branches();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_delete_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Switch to Deleting mode - actual delete happens on next frame
                self.mode = AppMode::Deleting;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    fn select_next(&mut self) {
        if self.worktrees.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.worktrees.len() - 1);
        self.list_state.select(Some(self.selected));
        self.load_status_content();
    }

    fn select_previous(&mut self) {
        if self.worktrees.is_empty() {
            return;
        }
        self.selected = self.selected.saturating_sub(1);
        self.list_state.select(Some(self.selected));
        self.load_status_content();
    }

    fn start_create(&mut self) {
        self.mode = AppMode::Creating;
        self.input.clear();
        self.input_cursor = 0;
        self.update_filtered_branches();
    }

    fn start_delete(&mut self) {
        if let Some(wt) = self.worktrees.get(self.selected) {
            if wt.is_main {
                self.error = Some("Cannot delete main worktree".to_string());
                return;
            }
            self.mode = AppMode::ConfirmDelete;
        }
    }

    fn refresh_worktrees(&mut self) {
        match crate::git::list_worktrees(&self.repo_path) {
            Ok(worktrees) => {
                self.worktrees = worktrees;
                if self.selected >= self.worktrees.len() {
                    self.selected = self.worktrees.len().saturating_sub(1);
                }
                self.list_state.select(Some(self.selected));
                self.load_status_content();
            }
            Err(e) => {
                self.error = Some(format!("Failed to list worktrees: {}", e));
            }
        }
    }

    fn refresh_branches(&mut self) {
        match crate::git::list_branches(&self.repo_path) {
            Ok(branches) => {
                self.branches = branches;
            }
            Err(e) => {
                self.error = Some(format!("Failed to list branches: {}", e));
            }
        }
    }

    fn update_filtered_branches(&mut self) {
        if self.input.is_empty() {
            self.filtered_branches = self.branches.clone();
        } else {
            let input_lower = self.input.to_lowercase();
            self.filtered_branches = self
                .branches
                .iter()
                .filter(|b| b.to_lowercase().contains(&input_lower))
                .cloned()
                .collect();
        }
    }

    fn load_status_content(&mut self) {
        if let Some(wt) = self.worktrees.get(self.selected) {
            match self.detail_view {
                DetailViewMode::Notes => {
                    let status_path = wt.path.join(".worktree-status.md");
                    if status_path.exists() {
                        self.status_content = std::fs::read_to_string(&status_path).ok();
                    } else {
                        self.status_content = None;
                    }
                }
                DetailViewMode::GitStatus => {
                    self.status_content = crate::git::get_git_status(&wt.path).ok();
                }
            }
        } else {
            self.status_content = None;
        }
    }

    fn toggle_detail_view(&mut self) {
        self.detail_view = match self.detail_view {
            DetailViewMode::Notes => DetailViewMode::GitStatus,
            DetailViewMode::GitStatus => DetailViewMode::Notes,
        };
        self.load_status_content();
    }

    fn create_worktree(&mut self) -> Result<()> {
        let branch = self.input.trim().to_string();
        if branch.is_empty() {
            return Ok(());
        }

        // Check if branch already exists
        let branch_exists = self.branches.contains(&branch);

        // Generate worktree path
        let repo_name = self
            .repo_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo");
        let worktree_path = self
            .repo_path
            .parent()
            .unwrap_or(&self.repo_path)
            .join(format!("{}-{}", repo_name, branch.replace('/', "-")));

        // Create worktree
        match crate::git::create_worktree(&self.repo_path, &branch, &worktree_path, branch_exists) {
            Ok(()) => {
                // Generate status file
                let status_content = crate::status::generate_status_file(&branch);
                let status_path = worktree_path.join(".worktree-status.md");
                let _ = std::fs::write(&status_path, status_content);

                // Run init script if exists
                let init_script = self.repo_path.join(".worktree-init.sh");
                if init_script.exists() {
                    let _ = std::process::Command::new("sh")
                        .arg(&init_script)
                        .arg(&worktree_path)
                        .current_dir(&worktree_path)
                        .status();
                }

                // Reset state and refresh
                self.mode = AppMode::Normal;
                self.input.clear();
                self.input_cursor = 0;
                self.refresh_worktrees();
                self.refresh_branches();
            }
            Err(e) => {
                self.error = Some(format!("Failed to create worktree: {}", e));
                self.mode = AppMode::Normal;
            }
        }

        Ok(())
    }

    fn delete_worktree(&mut self) -> Result<()> {
        if let Some(wt) = self.worktrees.get(self.selected) {
            if wt.is_main {
                self.error = Some("Cannot delete main worktree".to_string());
                self.mode = AppMode::Normal;
                return Ok(());
            }

            let path = wt.path.clone();
            match crate::git::delete_worktree(&self.repo_path, &path, wt.has_changes) {
                Ok(()) => {
                    self.mode = AppMode::Normal;
                    self.refresh_worktrees();
                }
                Err(e) => {
                    self.error = Some(format!("Failed to delete worktree: {}", e));
                    self.mode = AppMode::Normal;
                }
            }
        }

        Ok(())
    }

    fn open_editor(&mut self) -> Result<()> {
        if let Some(wt) = self.worktrees.get(self.selected) {
            let status_path = wt.path.join(".worktree-status.md");

            // Create status file if it doesn't exist
            if !status_path.exists() {
                let branch = wt.branch.as_deref().unwrap_or("unknown");
                let content = crate::status::generate_status_file(branch);
                std::fs::write(&status_path, content)?;
            }

            // Get editor from environment
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

            // Restore terminal for editor
            ratatui::restore();

            // Run editor
            let status = std::process::Command::new(&editor)
                .arg(&status_path)
                .status();

            // Reinitialize terminal
            let _ = ratatui::init();

            if let Err(e) = status {
                self.error = Some(format!("Failed to open editor: {}", e));
            }

            self.load_status_content();
            self.refresh_worktrees();
            self.needs_full_redraw = true;
        }

        Ok(())
    }

    fn merge_main(&mut self) -> Result<()> {
        if let Some(wt) = self.worktrees.get(self.selected) {
            if wt.is_main {
                self.error = Some("Cannot merge main into itself".to_string());
                return Ok(());
            }

            match crate::git::merge_main_ff(&wt.path) {
                Ok(()) => {
                    self.refresh_worktrees();
                    self.load_status_content();
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                }
            }
        }

        Ok(())
    }

    fn exit_to_worktree(&mut self) {
        if let Some(wt) = self.worktrees.get(self.selected) {
            self.exit_path = Some(wt.path.clone());
            self.should_quit = true;
        }
    }

    pub fn selected_worktree(&self) -> Option<&Worktree> {
        self.worktrees.get(self.selected)
    }
}
