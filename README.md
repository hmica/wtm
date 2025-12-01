# wtm - Worktree Manager

A terminal UI for Git worktree management, inspired by lazygit.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **List & navigate** worktrees with keyboard
- **Create worktrees** from existing or new branches (with autocomplete)
- **Delete worktrees** with confirmation and safety warnings
- **Status tracking** via `.worktree-status.md` files with progress indicators
- **Git integration**: see commits ahead/behind main, dirty state
- **Merged indicator**: green checkmark shows worktrees ready to delete
- **Toggle views**: switch between notes and `git status` output
- **Custom shortcuts**: configure your own keybindings and commands
- **Quick actions**: open lazygit, IDE, or merge main with one key

## Installation

### From source

```bash
git clone https://github.com/hmica/wtm.git
cd wtm
cargo build --release
cargo install --path .
```

### Just build (without installing)

```bash
cargo build --release
./target/release/wtm
```

## Usage

```bash
# Run in any git repository
wtm
```

### Shell Integration

Add to your `~/.bashrc` or `~/.zshrc` for `cd` on Enter:

```bash
wtm() {
    local dir
    dir=$(command wtm "$@")
    if [[ -n "$dir" && -d "$dir" ]]; then
        cd "$dir"
    fi
}
```

## Keybindings

### Navigation
| Key | Action |
|-----|--------|
| `j` / `k` | Move up/down |
| `Enter` | Exit and cd to worktree |
| `t` / `Tab` | Toggle notes/git status view |

### Actions
| Key | Action |
|-----|--------|
| `n` | Create new worktree |
| `d` | Delete worktree |
| `e` | Edit status file in `$EDITOR` |
| `g` | Open lazygit |
| `c` | Open in IDE (`$CODE_IDE`, defaults to `code`) |
| `m` | Merge main (fast-forward only) |
| `r` | Refresh list |

### Other
| Key | Action |
|-----|--------|
| `?` | Help (shows all shortcuts from config) |
| `q` | Quit |

## List Indicators

```
  main (main)              [---]     # Main branch (green)
✓ feature-done             [3/3]     # Merged & clean (green) - ready to delete
* feature-wip        ↑3↓1  [1/5]     # Dirty + unmerged commits (yellow * / cyan)
  feature-clean      ↑2↓0  [2/4]     # Clean but unmerged (cyan)
```

- `✓` = merged (ahead=0) and clean - safe to delete
- `*` = uncommitted changes
- `↑N↓M` = commits ahead/behind main
- `[x/y]` = task progress from status file

## Configuration

wtm uses a config file at `~/.config/wtm/config.toml`. A default one is created on first run.

### Example config.toml

```toml
[shortcuts]
# Built-in actions
n = { action = "create" }
d = { action = "delete" }
e = { action = "edit" }
m = { action = "merge_main" }
t = { action = "toggle_view" }
r = { action = "refresh" }
"?" = { action = "help" }
q = { action = "quit" }
Enter = { action = "cd" }

# Custom commands
g = { cmd = "lazygit", mode = "replace" }
c = { cmd = "${CODE_IDE:-code} $1 $2", mode = "detach" }

# Add your own!
l = { cmd = "gh pr list", mode = "replace" }
p = { cmd = "gh pr create --web", mode = "detach" }
```

### Command Modes

| Mode | Behavior |
|------|----------|
| `replace` | Takes over terminal (like lazygit, vim) |
| `detach` | Spawns in background, keeps wtm running |

### Variables

| Variable | Description |
|----------|-------------|
| `$1` or `$path` | Worktree path |
| `$2` or `$branch` | Branch name |
| `$repo` | Main repo path |

### Built-in Actions

`create`, `delete`, `edit`, `merge_main`, `toggle_view`, `refresh`, `help`, `quit`, `cd`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EDITOR` | `vim` | Editor for status files |
| `CODE_IDE` | `code` | Used in default config for IDE command |

## Init Script

When creating a new worktree, wtm looks for `.worktree-init.sh` in your main repo. If found, it runs automatically in the new worktree directory.

**Example `.worktree-init.sh`:**

```bash
#!/bin/bash
# $1 = new worktree path

# Copy environment files
cp ../.env .env 2>/dev/null

# Install dependencies
pnpm install

# Setup database
pnpm db:migrate
```

Make it executable: `chmod +x .worktree-init.sh`

## Keeping Branches Up-to-Date

Press `m` to merge main into your current branch (fast-forward only). This:
1. Fetches latest `origin/main`
2. Merges with `--ff-only`
3. Shows error if fast-forward not possible (you may need to rebase)

## License

MIT
