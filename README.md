# wtm - Worktree Manager

A terminal UI for Git worktree management, inspired by lazygit.

![Rust](https://img.shields.io/badge/rust-stable-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- **List & navigate** worktrees with keyboard
- **Create worktrees** from existing or new branches (with autocomplete)
- **Delete worktrees** with confirmation
- **Status tracking** via `.worktree-status.md` files with progress indicators
- **Git integration**: see commits ahead/behind main, dirty state
- **Merged indicator**: green checkmark shows worktrees ready to delete
- **Toggle views**: switch between notes and `git status` output
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
| `?` | Help |
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

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `EDITOR` | `vim` | Editor for status files |
| `CODE_IDE` | `code` | IDE command (e.g., `cursor`, `zed`, `nvim`) |

## License

MIT
