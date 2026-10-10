use notify::{recommended_watcher, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    Modified,
    Deleted,
}

pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    watched_parent: Option<PathBuf>,
    watched_file: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_parent: None,
            watched_file: None,
        }
    }

    pub fn unwatch(&mut self) {
        if let (Some(mut watcher), Some(parent)) = (self.watcher.take(), self.watched_parent.take())
        {
            let _ = watcher.unwatch(&parent);
        }
        self.watched_file = None;
    }

    pub fn watch<F>(&mut self, file_path: &Path, on_change: F) -> Result<(), notify::Error>
    where
        F: Fn(FileChange) + Send + 'static,
    {
        self.unwatch();

        let parent = file_path.parent().unwrap_or(file_path).to_path_buf();
        let target_file = file_path.to_path_buf();
        let target_name = file_path.file_name().map(|n| n.to_os_string());

        let target_file_for_closure = target_file.clone();
        let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let matches_file = event.paths.iter().any(|p| {
                    p == &target_file_for_closure
                        || target_name
                            .as_ref()
                            .is_some_and(|name| p.file_name() == Some(name))
                });

                if !matches_file {
                    return;
                }

                if !target_file_for_closure.exists() {
                    on_change(FileChange::Deleted);
                } else {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            on_change(FileChange::Modified);
                        }
                        _ => {}
                    }
                }
            }
        })?;

        watcher.watch(&parent, RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);
        self.watched_parent = Some(parent);
        self.watched_file = Some(target_file);

        Ok(())
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_file_watcher_detects_changes() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("watch_test.md");
        fs::write(&file_path, "Initial content").unwrap();

        let (tx, rx) = mpsc::channel();
        let mut watcher = FileWatcher::new();
        watcher
            .watch(&file_path, move |change| {
                let _ = tx.send(change);
            })
            .unwrap();

        // 1. External modification
        fs::write(&file_path, "Modified content").unwrap();
        let change = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Did not receive modification event");
        assert_eq!(change, FileChange::Modified);

        std::thread::sleep(Duration::from_millis(50));
        while rx.try_recv().is_ok() {}

        // 2. External deletion
        fs::remove_file(&file_path).unwrap();
        let change = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("Did not receive deletion event");
        assert_eq!(change, FileChange::Deleted);
    }
}
