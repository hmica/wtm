use crate::git::WorktreeStatus;

const STATUS_TEMPLATE: &str = r#"# Worktree: {branch_name}

## Purpose
<!-- What this worktree is for -->


## Status
- [ ] Implementation complete
- [ ] Tests passing
- [ ] Ready for review

## Notes
<!-- Blockers, context -->


## Related
<!-- Issue #, PR # -->

"#;

pub fn generate_status_file(branch_name: &str) -> String {
    STATUS_TEMPLATE.replace("{branch_name}", branch_name)
}

pub fn parse_status_file(content: &str) -> WorktreeStatus {
    let mut status = WorktreeStatus::default();
    let mut checked = 0u32;
    let mut total = 0u32;

    // Parse checkboxes
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            checked += 1;
            total += 1;
        } else if trimmed.starts_with("- [ ]") {
            total += 1;
        }
    }

    status.progress = (checked, total);

    // Extract purpose (first non-empty, non-comment line after "## Purpose")
    let mut in_purpose = false;
    for line in content.lines() {
        if line.starts_with("## Purpose") {
            in_purpose = true;
            continue;
        }
        if in_purpose {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("<!--") {
                status.purpose = Some(trimmed.to_string());
                break;
            }
            if line.starts_with("## ") {
                break;
            }
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_progress() {
        let content = r#"# Worktree: test
## Status
- [x] Done
- [ ] Not done
- [X] Also done
"#;
        let status = parse_status_file(content);
        assert_eq!(status.progress, (2, 3));
    }

    #[test]
    fn test_parse_purpose() {
        let content = r#"# Worktree: test
## Purpose
Implement OAuth2 authentication

## Status
"#;
        let status = parse_status_file(content);
        assert_eq!(status.purpose, Some("Implement OAuth2 authentication".to_string()));
    }
}
