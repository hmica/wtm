use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

#[derive(Default)]
pub struct WorktreeStatus {
    pub purpose: Option<String>,
    pub progress: (u32, u32), // (checked, total)
}

pub struct Worktree {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_main: bool,
    #[allow(dead_code)]
    pub is_bare: bool,
    pub has_changes: bool,
    pub status: WorktreeStatus,
    pub ahead: u32,
    pub behind: u32,
}

pub fn list_worktrees(repo_path: &Path) -> Result<Vec<Worktree>> {
    let output = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git worktree list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_worktree_list(&stdout)
}

fn parse_worktree_list(output: &str) -> Result<Vec<Worktree>> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut current_commit = String::new();
    let mut current_branch: Option<String> = None;
    let mut is_bare = false;

    for line in output.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if exists
            if let Some(path) = current_path.take() {
                let has_changes = has_uncommitted_changes(&path).unwrap_or(false);
                let status = load_worktree_status(&path);
                let branch_ref = current_branch.as_deref();
                let (ahead, behind) = get_ahead_behind(&path, branch_ref);
                worktrees.push(Worktree {
                    path,
                    branch: current_branch.take(),
                    commit: std::mem::take(&mut current_commit),
                    is_main: worktrees.is_empty(),
                    is_bare,
                    has_changes,
                    status,
                    ahead,
                    behind,
                });
                is_bare = false;
            }
            current_path = Some(PathBuf::from(&line[9..]));
        } else if line.starts_with("HEAD ") {
            // Take first 7 chars for short hash
            current_commit = line[5..].chars().take(7).collect();
        } else if line.starts_with("branch ") {
            let branch = &line[7..];
            current_branch = Some(
                branch
                    .strip_prefix("refs/heads/")
                    .unwrap_or(branch)
                    .to_string(),
            );
        } else if line == "bare" {
            is_bare = true;
        }
    }

    // Don't forget the last worktree
    if let Some(path) = current_path {
        let has_changes = has_uncommitted_changes(&path).unwrap_or(false);
        let status = load_worktree_status(&path);
        let branch_ref = current_branch.as_deref();
        let (ahead, behind) = get_ahead_behind(&path, branch_ref);
        worktrees.push(Worktree {
            path,
            branch: current_branch,
            commit: current_commit,
            is_main: worktrees.is_empty(),
            is_bare,
            has_changes,
            status,
            ahead,
            behind,
        });
    }

    Ok(worktrees)
}

fn load_worktree_status(path: &Path) -> WorktreeStatus {
    let status_path = path.join(".worktree-status.md");
    if !status_path.exists() {
        return WorktreeStatus::default();
    }

    let content = match std::fs::read_to_string(&status_path) {
        Ok(c) => c,
        Err(_) => return WorktreeStatus::default(),
    };

    crate::status::parse_status_file(&content)
}

pub fn list_branches(repo_path: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["branch", "--list", "--format=%(refname:short)"])
        .current_dir(repo_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git branch list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

    Ok(branches)
}

pub fn create_worktree(
    repo_path: &Path,
    branch: &str,
    worktree_path: &Path,
    branch_exists: bool,
) -> Result<()> {
    let output = if branch_exists {
        // Use existing branch
        Command::new("git")
            .args([
                "worktree",
                "add",
                worktree_path.to_str().unwrap_or_default(),
                branch,
            ])
            .current_dir(repo_path)
            .output()?
    } else {
        // Create new branch
        Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                branch,
                worktree_path.to_str().unwrap_or_default(),
            ])
            .current_dir(repo_path)
            .output()?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git worktree add failed: {}", stderr));
    }

    Ok(())
}

pub fn delete_worktree(repo_path: &Path, worktree_path: &Path, force: bool) -> Result<()> {
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(worktree_path.to_str().unwrap_or_default());

    let output = Command::new("git").args(&args).current_dir(repo_path).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git worktree remove failed: {}", stderr));
    }

    Ok(())
}

pub fn has_uncommitted_changes(worktree_path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(worktree_path)
        .output()?;

    if !output.status.success() {
        return Ok(false);
    }

    Ok(!output.stdout.is_empty())
}

pub fn get_git_status(worktree_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["status", "--short"])
        .current_dir(worktree_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git status failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.is_empty() {
        Ok("Working tree clean".to_string())
    } else {
        Ok(stdout.to_string())
    }
}

/// Get commits ahead/behind compared to a base branch (main/master)
/// Returns (ahead, behind) tuple
pub fn get_ahead_behind(worktree_path: &Path, branch: Option<&str>) -> (u32, u32) {
    let branch = match branch {
        Some(b) => b,
        None => return (0, 0), // Detached HEAD
    };

    // Try to find the main branch name (main or master)
    let main_branch = find_main_branch(worktree_path).unwrap_or_else(|| "main".to_string());

    // Don't compare main to itself
    if branch == main_branch {
        return (0, 0);
    }

    // Use rev-list to count commits
    let output = Command::new("git")
        .args([
            "rev-list",
            "--left-right",
            "--count",
            &format!("{}...{}", main_branch, branch),
        ])
        .current_dir(worktree_path)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = stdout.trim().split('\t').collect();
            if parts.len() == 2 {
                let behind = parts[0].parse().unwrap_or(0);
                let ahead = parts[1].parse().unwrap_or(0);
                (ahead, behind)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}

fn find_main_branch(repo_path: &Path) -> Option<String> {
    // Check for common main branch names
    for name in &["main", "master"] {
        let output = Command::new("git")
            .args(["rev-parse", "--verify", &format!("refs/heads/{}", name)])
            .current_dir(repo_path)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Merge main/master into current branch using fast-forward only
/// Fetches from origin first to ensure we have the latest
pub fn merge_main_ff(worktree_path: &Path) -> Result<()> {
    let main_branch = find_main_branch(worktree_path)
        .ok_or_else(|| anyhow!("Could not find main/master branch"))?;

    // First fetch to ensure we have latest
    let _ = Command::new("git")
        .args(["fetch", "origin", &main_branch])
        .current_dir(worktree_path)
        .output();

    // Merge with ff-only
    let output = Command::new("git")
        .args(["merge", "--ff-only", &format!("origin/{}", main_branch)])
        .current_dir(worktree_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Fast-forward not possible: {}", stderr.trim()));
    }

    Ok(())
}
