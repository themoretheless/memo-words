//! Durable learning progress behind a small port.
//!
//! `ProgressState` is the versioned envelope the app persists between sessions;
//! `ProgressStore` is the port `App` depends on, so the composition root picks
//! the concrete storage (a JSON file normally, a no-op in benchmark mode) and
//! tests inject an in-memory double. Words are keyed by headword: the deck's
//! natural stable key across MongoDB, the fallback table, and restarts. If the
//! model ever grows synthetic IDs, `schema_version` gates that migration.
//!
//! Storage rules, in the same defensive spirit as the config parser:
//! - A missing or unreadable state file yields an empty default; persistence
//!   problems must never block startup or lose the overlay.
//! - A corrupt file is quarantined (renamed `<name>.corrupt`) instead of being
//!   silently overwritten, so the evidence survives for inspection.
//! - Saves write a temp file first, then rename it into place, so a crash
//!   mid-write can never truncate the previous good state.
//! - Timestamps are UTC unix seconds, not process-relative instants, so state
//!   survives restarts and wall-clock display changes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Version of the persisted state layout. Bump when the shape changes in a way
/// a future loader must branch on; unknown JSON fields are already tolerated,
/// so purely additive fields do not need a bump.
const SCHEMA_VERSION: u32 = 1;

/// Current wall-clock time as UTC unix seconds. A clock before the epoch (a
/// badly set machine) reads as 0 rather than panicking.
pub fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Per-word learning history. Today this is exposure counting - the foundation
/// the future review scheduler reads and extends (ease, due time, grades).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordProgress {
    /// How many times the word has been shown across all sessions.
    pub exposures: u32,
    /// Unix seconds when the word was first ever shown.
    pub first_seen_unix: i64,
    /// Unix seconds when the word was most recently shown.
    pub last_seen_unix: i64,
}

/// The whole persisted learning state: a versioned envelope around per-word
/// progress. `BTreeMap` keeps serialization deterministic, so saved files diff
/// cleanly and tests can compare bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressState {
    pub schema_version: u32,
    /// The app version that last wrote the file; diagnostic breadcrumb only.
    #[serde(default)]
    pub app_version: String,
    #[serde(default)]
    pub words: BTreeMap<String, WordProgress>,
}

impl Default for ProgressState {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            words: BTreeMap::new(),
        }
    }
}

impl ProgressState {
    /// Record one exposure of `word` at `now_unix`. First sighting initialises
    /// both timestamps; repeats increment the count (saturating, so a corrupt
    /// or ancient counter can never wrap) and refresh only `last_seen_unix`.
    pub fn record_exposure(&mut self, word: &str, now_unix: i64) {
        self.words
            .entry(word.to_string())
            .and_modify(|p| {
                p.exposures = p.exposures.saturating_add(1);
                p.last_seen_unix = now_unix;
            })
            .or_insert(WordProgress {
                exposures: 1,
                first_seen_unix: now_unix,
                last_seen_unix: now_unix,
            });
    }

    /// Total exposures across every tracked word, for diagnostics.
    pub fn total_exposures(&self) -> u64 {
        self.words.values().map(|p| u64::from(p.exposures)).sum()
    }
}

/// The persistence port. `load` is infallible by design (any failure means
/// "start fresh"); `save` reports failure so the caller can log it.
pub trait ProgressStore {
    fn load(&self) -> ProgressState;
    fn save(&self, state: &ProgressState) -> std::io::Result<()>;
}

/// A store that keeps nothing. Benchmark mode uses it so measurement runs stay
/// hermetic, and it doubles as the fallback when no writable path exists.
pub struct NullProgressStore;

impl ProgressStore for NullProgressStore {
    fn load(&self) -> ProgressState {
        ProgressState::default()
    }

    fn save(&self, _state: &ProgressState) -> std::io::Result<()> {
        Ok(())
    }
}

/// JSON-file-backed store implementing the durability rules in the module doc.
pub struct FileProgressStore {
    path: PathBuf,
}

impl FileProgressStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Move a corrupt state file aside as `<name>.corrupt` so the next save
    /// does not silently destroy it. Best-effort: a failed rename only means
    /// the corrupt file stays in place until the next successful save.
    fn quarantine(&self) {
        let mut quarantined = self.path.as_os_str().to_owned();
        quarantined.push(".corrupt");
        if std::fs::rename(&self.path, PathBuf::from(&quarantined)).is_ok() {
            eprintln!(
                "memo-words: progress state was unreadable; moved it to {} and started fresh.",
                PathBuf::from(quarantined).display()
            );
        }
    }
}

impl ProgressStore for FileProgressStore {
    fn load(&self) -> ProgressState {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(text) => text,
            // Missing file is the normal first run; anything else (permissions,
            // I/O) also starts fresh rather than blocking startup.
            Err(_) => return ProgressState::default(),
        };
        match serde_json::from_str(&text) {
            Ok(state) => state,
            Err(_) => {
                self.quarantine();
                ProgressState::default()
            }
        }
    }

    fn save(&self, state: &ProgressState) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp = temp_sibling(&self.path);
        write_private(&tmp, &json)?;
        // The rename is the atomic commit: readers see either the old complete
        // file or the new complete file, never a partial write.
        std::fs::rename(&tmp, &self.path).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}

/// A temp path next to `path` (same filesystem, so the rename stays atomic),
/// unique per process so two instances cannot clobber each other's temp file
/// mid-write. Racing *saves* still last-write-wins on the final rename; a
/// single-instance lock is a separate, tracked concern.
fn temp_sibling(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".tmp.{}", std::process::id()));
    PathBuf::from(tmp)
}

/// Write `contents` to a fresh file readable only by the user (0o600 on unix),
/// since learning history is personal data.
fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(contents.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique throwaway directory per test, under the system temp root.
    fn scratch_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "memo-words-store-test-{}-{tag}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn record_exposure_initialises_and_increments() {
        let mut state = ProgressState::default();
        state.record_exposure("time", 100);
        assert_eq!(
            state.words["time"],
            WordProgress {
                exposures: 1,
                first_seen_unix: 100,
                last_seen_unix: 100,
            }
        );

        state.record_exposure("time", 250);
        let p = &state.words["time"];
        assert_eq!(p.exposures, 2);
        assert_eq!(p.first_seen_unix, 100, "first sighting must be preserved");
        assert_eq!(p.last_seen_unix, 250);

        state.record_exposure("be", 300);
        assert_eq!(state.words.len(), 2);
        assert_eq!(state.total_exposures(), 3);
    }

    #[test]
    fn exposure_count_saturates_instead_of_wrapping() {
        let mut state = ProgressState::default();
        state.record_exposure("word", 1);
        state.words.get_mut("word").unwrap().exposures = u32::MAX;
        state.record_exposure("word", 2);
        assert_eq!(state.words["word"].exposures, u32::MAX);
    }

    #[test]
    fn file_store_round_trips_state() {
        let dir = scratch_dir("round-trip");
        let store = FileProgressStore::new(dir.join("progress.json"));

        let mut state = ProgressState::default();
        state.record_exposure("time", 100);
        state.record_exposure("be", 150);
        store.save(&state).unwrap();

        assert_eq!(store.load(), state);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_file_loads_the_empty_default() {
        let dir = scratch_dir("missing");
        let store = FileProgressStore::new(dir.join("progress.json"));
        assert_eq!(store.load(), ProgressState::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_file_is_quarantined_and_load_starts_fresh() {
        let dir = scratch_dir("corrupt");
        let path = dir.join("progress.json");
        std::fs::write(&path, "{ not valid json").unwrap();

        let store = FileProgressStore::new(path.clone());
        assert_eq!(store.load(), ProgressState::default());
        assert!(
            !path.exists(),
            "corrupt file must be moved aside, not left in place"
        );
        assert!(
            dir.join("progress.json.corrupt").exists(),
            "corrupt file must be preserved for inspection"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_fields_are_tolerated_for_forward_compatibility() {
        // A file written by a NEWER app version with extra fields must still
        // load; serde ignores unknown fields by default and this test guards
        // against someone adding deny_unknown_fields later.
        let dir = scratch_dir("forward-compat");
        let path = dir.join("progress.json");
        std::fs::write(
            &path,
            r#"{
                "schema_version": 1,
                "app_version": "9.9.9",
                "future_field": {"anything": true},
                "words": {
                    "time": {
                        "exposures": 3,
                        "first_seen_unix": 10,
                        "last_seen_unix": 20,
                        "future_grade": 4
                    }
                }
            }"#,
        )
        .unwrap();

        let state = FileProgressStore::new(path).load();
        assert_eq!(state.words["time"].exposures, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_parent_directories_and_leaves_no_temp_file() {
        let dir = scratch_dir("mkdir");
        let path = dir.join("nested").join("deeper").join("progress.json");
        let store = FileProgressStore::new(path.clone());

        store.save(&ProgressState::default()).unwrap();
        assert!(path.exists());
        let siblings: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(siblings, vec![std::ffi::OsString::from("progress.json")]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn saved_state_is_readable_only_by_the_user() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch_dir("mode");
        let path = dir.join("progress.json");
        FileProgressStore::new(path.clone())
            .save(&ProgressState::default())
            .unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "state file must be user-only");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn null_store_loads_default_and_saves_nothing() {
        let store = NullProgressStore;
        assert_eq!(store.load(), ProgressState::default());
        assert!(store.save(&ProgressState::default()).is_ok());
    }
}
