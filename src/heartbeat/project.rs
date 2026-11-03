use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Names the project an entity belongs to: the basename of its git
/// repository root, or of its directory when it is not inside a repository.
/// An entity that is not a path (an app name) falls back to `cwd`.
pub fn detect(entity: &Path, cwd: &Path) -> Option<String> {
    let dir = anchor(entity, cwd);
    let root = git_toplevel(&dir).unwrap_or(dir);
    root.file_name().map(|n| n.to_string_lossy().into_owned())
}

fn anchor(entity: &Path, cwd: &Path) -> PathBuf {
    if entity.is_absolute() && entity.is_dir() {
        return entity.to_path_buf();
    }
    match entity.parent() {
        Some(parent) if entity.is_absolute() && parent.is_dir() => parent.to_path_buf(),
        _ => cwd.to_path_buf(),
    }
}

fn git_toplevel(dir: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--show-toplevel"])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let top = String::from_utf8(out.stdout).ok()?;
    let top = top.trim();
    (out.status.success() && !top.is_empty()).then(|| PathBuf::from(top))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sablier-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        dir
    }

    #[test]
    fn a_file_in_a_repository_belongs_to_the_repository_root() {
        let repo = scratch("repo");
        let status = Command::new("git")
            .args(["init", "-q"])
            .arg(&repo)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .status()
            .unwrap();
        assert!(status.success());
        let file = repo.join("src").join("main.rs");
        let expected = repo.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(detect(&file, Path::new("/")), Some(expected.clone()));
        assert_eq!(
            detect(Path::new("Firefox"), &repo.join("src")),
            Some(expected)
        );
        std::fs::remove_dir_all(&repo).unwrap();
    }

    #[test]
    fn outside_a_repository_the_directory_name_is_used() {
        let plain = scratch("plain");
        let dir = plain.join("src");
        let inside_repo = git_toplevel(&dir).is_some();
        let found = detect(&dir.join("x.txt"), Path::new("/"));
        std::fs::remove_dir_all(&plain).unwrap();
        if !inside_repo {
            assert_eq!(found, Some("src".into()));
        }
    }
}
