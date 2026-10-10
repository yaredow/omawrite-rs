use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub fn save_atomic(path: &Path, content: &str) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(content.as_bytes())?;
    temp.flush()?;
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

pub fn load_file(path: &Path) -> io::Result<String> {
    fs::read_to_string(path)
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct RecoverySnapshot {
    pub file_path: Option<PathBuf>,
    pub text: String,
}

pub struct RecoveryLock {
    _guard: fd_lock::RwLockWriteGuard<'static, File>,
    _lock: Box<fd_lock::RwLock<File>>,
}

impl RecoveryLock {
    pub fn try_acquire(lock_path: &Path) -> Option<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(lock_path)
            .ok()?;
        let mut boxed = Box::new(fd_lock::RwLock::new(file));
        let ptr: *mut fd_lock::RwLock<File> = &mut *boxed;
        // Safety: boxed is pinned on the heap and dropped after _guard
        let guard = unsafe { (*ptr).try_write().ok()? };
        Some(Self {
            _guard: guard,
            _lock: boxed,
        })
    }
}

pub struct RecoveryManager {
    pub _slot: usize,
    pub json_path: PathBuf,
    pub recovered_snapshot: Option<RecoverySnapshot>,
    _lock: RecoveryLock,
}

impl RecoveryManager {
    pub fn new() -> Option<Self> {
        Self::with_dir(glib::user_state_dir().join("omawrite-rs"))
    }

    pub fn with_dir(state_dir: PathBuf) -> Option<Self> {
        fs::create_dir_all(&state_dir).ok()?;

        for slot in 0..100 {
            let json_path = state_dir.join(format!("recovery-{slot}.json"));
            let lock_path = state_dir.join(format!("recovery-{slot}.lock"));

            if json_path.exists() {
                if let Some(lock) = RecoveryLock::try_acquire(&lock_path) {
                    let recovered_snapshot = fs::read_to_string(&json_path)
                        .ok()
                        .and_then(|data| serde_json::from_str::<RecoverySnapshot>(&data).ok());

                    return Some(Self {
                        _slot: slot,
                        json_path,
                        recovered_snapshot,
                        _lock: lock,
                    });
                }
            }
        }

        for slot in 0..100 {
            let json_path = state_dir.join(format!("recovery-{slot}.json"));
            let lock_path = state_dir.join(format!("recovery-{slot}.lock"));

            if !json_path.exists() {
                if let Some(lock) = RecoveryLock::try_acquire(&lock_path) {
                    return Some(Self {
                        _slot: slot,
                        json_path,
                        recovered_snapshot: None,
                        _lock: lock,
                    });
                }
            }
        }

        None
    }

    pub fn write_draft(&self, file_path: Option<&Path>, text: &str) -> io::Result<()> {
        let snapshot = RecoverySnapshot {
            file_path: file_path.map(Path::to_path_buf),
            text: text.to_string(),
        };
        let json = serde_json::to_string(&snapshot).map_err(io::Error::other)?;
        save_atomic(&self.json_path, &json)
    }

    pub fn clear(&self) {
        let _ = fs::remove_file(&self.json_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_atomic_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("test_document.md");
        let content = "# Test Header\n\nSome test content.";

        save_atomic(&target, content).unwrap();
        let loaded = load_file(&target).unwrap();
        assert_eq!(loaded, content);
    }

    #[test]
    fn test_recovery_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let state_dir = dir.path().to_path_buf();

        let mgr = RecoveryManager::with_dir(state_dir.clone()).unwrap();
        assert!(mgr.recovered_snapshot.is_none());

        mgr.write_draft(Some(Path::new("/tmp/test.md")), "Unsaved draft content")
            .unwrap();
        assert!(mgr.json_path.exists());

        // Simulate crash by dropping manager without clear()
        drop(mgr);

        // Next session reclaims the orphaned draft
        let recovered_mgr = RecoveryManager::with_dir(state_dir.clone()).unwrap();
        let snapshot = recovered_mgr.recovered_snapshot.as_ref().unwrap();
        assert_eq!(snapshot.text, "Unsaved draft content");
        assert_eq!(
            snapshot.file_path.as_deref(),
            Some(Path::new("/tmp/test.md"))
        );

        // Clean save clears the draft
        recovered_mgr.clear();
        assert!(!recovered_mgr.json_path.exists());
    }
}
