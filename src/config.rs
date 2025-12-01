use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CommandMode {
    Replace, // Take over terminal (like lazygit)
    Detach,  // Spawn in background (like IDE)
}

impl Default for CommandMode {
    fn default() -> Self {
        Self::Replace
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shortcut {
    BuiltIn {
        action: String,
    },
    Command {
        cmd: String,
        #[serde(default)]
        mode: CommandMode,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_shortcuts")]
    pub shortcuts: HashMap<String, Shortcut>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shortcuts: default_shortcuts(),
        }
    }
}

fn default_shortcuts() -> HashMap<String, Shortcut> {
    let mut shortcuts = HashMap::new();

    // Built-in actions
    shortcuts.insert("n".to_string(), Shortcut::BuiltIn { action: "create".to_string() });
    shortcuts.insert("d".to_string(), Shortcut::BuiltIn { action: "delete".to_string() });
    shortcuts.insert("e".to_string(), Shortcut::BuiltIn { action: "edit".to_string() });
    shortcuts.insert("m".to_string(), Shortcut::BuiltIn { action: "merge_main".to_string() });
    shortcuts.insert("t".to_string(), Shortcut::BuiltIn { action: "toggle_view".to_string() });
    shortcuts.insert("r".to_string(), Shortcut::BuiltIn { action: "refresh".to_string() });
    shortcuts.insert("?".to_string(), Shortcut::BuiltIn { action: "help".to_string() });
    shortcuts.insert("q".to_string(), Shortcut::BuiltIn { action: "quit".to_string() });
    shortcuts.insert("Enter".to_string(), Shortcut::BuiltIn { action: "cd".to_string() });

    // Default custom commands
    shortcuts.insert("g".to_string(), Shortcut::Command {
        cmd: "lazygit".to_string(),
        mode: CommandMode::Replace,
    });
    shortcuts.insert("c".to_string(), Shortcut::Command {
        cmd: "${CODE_IDE:-code} $1 $2".to_string(),
        mode: CommandMode::Detach,
    });

    shortcuts
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create config directory if needed
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;

        // Add header comment
        let content_with_header = format!(
r#"# wtm configuration file
#
# Shortcuts can be:
#   Built-in actions: {{ action = "create" }}
#   Custom commands:  {{ cmd = "lazygit", mode = "replace" }}
#
# Modes:
#   replace - takes over terminal (like lazygit, vim)
#   detach  - spawns in background (like VS Code)
#
# Variables for commands:
#   $1 or $path   - worktree path
#   $2 or $branch - branch name
#   $repo         - main repo path
#
# Built-in actions:
#   create, delete, edit, merge_main, toggle_view, refresh, help, quit, cd

{}"#, content);

        fs::write(&config_path, content_with_header)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        // Use XDG_CONFIG_HOME or ~/.config (not macOS ~/Library/Application Support)
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".config")
            });
        Ok(config_dir.join("wtm").join("config.toml"))
    }

    pub fn get_shortcut(&self, key: &str) -> Option<&Shortcut> {
        self.shortcuts.get(key)
    }
}
