use std::process::Command;
use std::path::Path;

use compact_str::CompactString;

#[derive(Debug, Clone)]
pub struct GitDiffLine {
    pub content: CompactString,
    pub line_type: DiffLineType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineType {
    Added,
    Removed,
    Unchanged,
    Header,
}

#[derive(Debug, Clone)]
pub struct GitFileChange {
    pub path: CompactString,
    pub status: CompactString,
    pub diff: Vec<GitDiffLine>,
    pub is_expanded: bool,
}

pub struct GitManager {
    pub root: Option<std::path::PathBuf>,
}

impl GitManager {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn set_root(&mut self, path: &Path) {
        self.root = Some(path.to_path_buf());
    }

    pub fn get_changes(&self) -> Vec<GitFileChange> {
        let root = match &self.root {
            Some(r) => r,
            None => return vec![],
        };

        // Get git status
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .arg("-uall")
            .current_dir(root)
            .output()
            .ok();

        let mut changes = vec![];

        if let Some(output) = output {
            let stdout = CompactString::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.len() < 4 { continue; }
                let status = CompactString::from(line[..2].trim());
                let path = CompactString::from(line[3..].trim().trim_matches('"'));

                let diff = self.get_diff_for_file(Path::new(&path));
                changes.push(GitFileChange {
                    path,
                    status,
                    diff,
                    is_expanded: true,
                });
            }
        }

        changes
    }

    fn get_diff_for_file(&self, path: &Path) -> Vec<GitDiffLine> {
        let root = match &self.root {
            Some(r) => r,
            None => return vec![],
        };

        // Try git diff HEAD first (for tracked files, staged or unstaged)
        let output = Command::new("git")
            .arg("diff")
            .arg("HEAD")
            .arg("--")
            .arg(path)
            .current_dir(root)
            .output()
            .ok();

        let mut diff_lines = vec![];

        if let Some(output) = output {
            let stdout = CompactString::from_utf8_lossy(&output.stdout);
            if !stdout.is_empty() {
                for line in stdout.lines() {
                    let line_type = if line.starts_with('+') && !line.starts_with("+++") {
                        DiffLineType::Added
                    } else if line.starts_with('-') && !line.starts_with("---") {
                        DiffLineType::Removed
                    } else if line.starts_with("@@") {
                        DiffLineType::Header
                    } else {
                        DiffLineType::Unchanged
                    };

                    diff_lines.push(GitDiffLine {
                        content: line.into(),
                        line_type,
                    });
                }
                return diff_lines;
            }
        }

        // If diff was empty, maybe it's untracked?
        let full_path = std::path::Path::new(root).join(path);
        if let Ok(content) = std::fs::read_to_string(full_path) {
            diff_lines.push(GitDiffLine {
                content: format!("--- /dev/null").into(),
                line_type: DiffLineType::Header,
            });
            diff_lines.push(GitDiffLine {
                content: format!("+++ b/{}", path.display()).into(),
                line_type: DiffLineType::Header,
            });
            for line in content.lines() {
                diff_lines.push(GitDiffLine {
                    content: format!("+{}", line).into(),
                    line_type: DiffLineType::Added,
                });
            }
        }

        diff_lines
    }
}
