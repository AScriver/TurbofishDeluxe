//! Offline checks for the existing pending-file publication contract.

use serde::{Serialize, Serializer, ser::SerializeStruct};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use turbofish_deluxe::{
    persistence::{io_error, write_json},
    settings::{GameSettings, SETTINGS_FORMAT_VERSION, SettingsSave},
};

const OLD_GENERATION: &[u8] = b"{\"tick\":1}";
const NEW_GENERATION: &[u8] = b"{\n  \"tick\": 2\n}";
static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct FixtureDirectory {
    root: PathBuf,
    path: PathBuf,
}

impl FixtureDirectory {
    fn new(label: &str) -> Self {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".scratch/checkpoint-error-context/fixtures/writer");
        fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let path = root.join(format!(
            "{label}-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self { root, path }
    }

    fn checkpoint(&self) -> PathBuf {
        self.path.join("state.local.json")
    }
}

impl Drop for FixtureDirectory {
    fn drop(&mut self) {
        let resolved = self.path.canonicalize().unwrap();
        assert!(resolved.starts_with(&self.root) && resolved != self.root);
        fs::remove_dir_all(resolved).unwrap();
    }
}

#[test]
fn checkpoint_replacement_keeps_pretty_json_and_extension_replacement() {
    let directory = FixtureDirectory::new("replacement");
    let path = directory.checkpoint();
    let pending = directory.path.join("state.local.pending");
    fs::write(&path, OLD_GENERATION).unwrap();
    fs::write(&pending, b"stale pending generation").unwrap();

    write_json(&path, &serde_json::json!({ "tick": 2 })).unwrap();

    assert_eq!(fs::read(&path).unwrap(), NEW_GENERATION);
    assert!(!pending.exists());
    assert!(!directory.path.join("state.local.json.pending").exists());
}

struct FailingSerialization;

impl Serialize for FailingSerialization {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut object = serializer.serialize_struct("FailingSerialization", 2)?;
        object.serialize_field("tick", &2)?;
        Err(serde::ser::Error::custom("fixture serialization failure"))
    }
}

#[test]
fn serialization_failure_leaves_current_and_existing_pending_bytes_unchanged() {
    let directory = FixtureDirectory::new("serialization-failure");
    let path = directory.checkpoint();
    let pending = path.with_extension("pending");
    fs::write(&path, OLD_GENERATION).unwrap();
    fs::write(&pending, b"prior pending bytes").unwrap();

    let error = write_json(&path, &FailingSerialization).unwrap_err();

    assert!(error.downcast_ref::<serde_json::Error>().is_some());
    assert!(io_error(error.as_ref()).is_none());
    assert_eq!(error.to_string(), "fixture serialization failure");
    assert_eq!(fs::read(&path).unwrap(), OLD_GENERATION);
    assert_eq!(fs::read(&pending).unwrap(), b"prior pending bytes");
}

#[test]
fn failed_pending_write_preserves_current_and_reports_io_error() {
    let directory = FixtureDirectory::new("pending-write-failure");
    let path = directory.checkpoint();
    let pending = path.with_extension("pending");
    fs::write(&path, OLD_GENERATION).unwrap();
    fs::create_dir(&pending).unwrap();
    fs::write(pending.join("marker"), b"unrelated bytes").unwrap();

    let error = write_json(&path, &serde_json::json!({ "tick": 2 })).unwrap_err();

    assert!(error.downcast_ref::<std::io::Error>().is_some());
    assert!(std::ptr::eq(
        io_error(error.as_ref()).unwrap(),
        error.downcast_ref::<std::io::Error>().unwrap()
    ));
    assert_eq!(fs::read(&path).unwrap(), OLD_GENERATION);
    assert_eq!(
        fs::read(pending.join("marker")).unwrap(),
        b"unrelated bytes"
    );
}

#[test]
fn writer_does_not_create_missing_parent_directories() {
    let directory = FixtureDirectory::new("missing-parent");
    let path = directory.path.join("missing/state.json");

    let error = write_json(&path, &serde_json::json!({ "tick": 2 })).unwrap_err();

    assert_eq!(
        io_error(error.as_ref()).unwrap().kind(),
        std::io::ErrorKind::NotFound
    );
    assert!(!path.parent().unwrap().exists());
}

#[test]
fn publication_failure_retains_pending_and_destination_with_path_context() {
    let directory = FixtureDirectory::new("publication-failure");
    let path = directory.checkpoint();
    fs::create_dir(&path).unwrap();
    fs::write(path.join("marker"), b"unrelated bytes").unwrap();

    let error = write_json(&path, &serde_json::json!({ "tick": 2 })).unwrap_err();

    assert!(
        error
            .to_string()
            .starts_with(&format!("Cannot publish checkpoint {}: ", path.display()))
    );
    let source = error
        .source()
        .unwrap()
        .downcast_ref::<std::io::Error>()
        .unwrap();
    assert!(std::ptr::eq(io_error(error.as_ref()).unwrap(), source));
    assert_eq!(fs::read(path.join("marker")).unwrap(), b"unrelated bytes");
    assert_eq!(
        fs::read(path.with_extension("pending")).unwrap(),
        NEW_GENERATION
    );
}

#[test]
fn settings_save_creates_parent_and_replaces_validated_settings() {
    let directory = FixtureDirectory::new("settings-replacement");
    let path = directory.path.join("options/settings.json");
    let mut settings = SettingsSave {
        format_version: SETTINGS_FORMAT_VERSION,
        settings: GameSettings::default(),
    };
    settings.save(&path).unwrap();
    settings.settings.music_volume = 0.25;
    settings.settings.custom_cursor = true;
    settings.save(&path).unwrap();

    let reopened = SettingsSave::load_or_default(&path).unwrap();
    assert_eq!(reopened.format_version, SETTINGS_FORMAT_VERSION);
    assert_eq!(reopened.settings, settings.settings);
    assert!(!path.with_extension("pending").exists());
}

#[test]
fn invalid_settings_leave_current_and_pending_bytes_unchanged() {
    let directory = FixtureDirectory::new("settings-validation");
    let path = directory.checkpoint();
    let pending = path.with_extension("pending");
    fs::write(&path, OLD_GENERATION).unwrap();
    fs::write(&pending, b"prior pending bytes").unwrap();
    let mut settings = SettingsSave {
        format_version: SETTINGS_FORMAT_VERSION,
        settings: GameSettings {
            music_volume: f32::NAN,
            ..GameSettings::default()
        },
    };
    assert_eq!(
        settings.save(&path).unwrap_err().to_string(),
        "option volume must be between zero and one"
    );
    settings.settings = GameSettings::default();
    settings.format_version = SETTINGS_FORMAT_VERSION + 1;
    assert_eq!(
        settings.save(&path).unwrap_err().to_string(),
        "unsupported settings format"
    );
    assert_eq!(fs::read(&path).unwrap(), OLD_GENERATION);
    assert_eq!(fs::read(&pending).unwrap(), b"prior pending bytes");
}

#[derive(Debug)]
struct FixtureErrorContext(Box<dyn std::error::Error>);

impl std::fmt::Display for FixtureErrorContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "fixture context: {}", self.0)
    }
}

impl std::error::Error for FixtureErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.0.as_ref())
    }
}

#[test]
fn io_error_inspects_direct_and_nested_sources_without_inventing_os_codes() {
    for raw_code in [5, 32, 33] {
        let direct = std::io::Error::from_raw_os_error(raw_code);
        assert!(std::ptr::eq(io_error(&direct).unwrap(), &direct));
        let expected_kind = direct.kind();
        let contextual = FixtureErrorContext(Box::new(FixtureErrorContext(Box::new(direct))));
        let underlying = io_error(&contextual).unwrap();
        assert_eq!(underlying.kind(), expected_kind);
        assert_eq!(underlying.raw_os_error(), Some(raw_code));
    }

    let unclassified = std::io::Error::new(std::io::ErrorKind::Interrupted, "fixture interruption");
    let underlying = io_error(&unclassified).unwrap();
    assert!(std::ptr::eq(underlying, &unclassified));
    assert_eq!(underlying.kind(), std::io::ErrorKind::Interrupted);
    assert_eq!(underlying.raw_os_error(), None);
    assert!(io_error(&std::fmt::Error).is_none());
    assert!(io_error(&FixtureErrorContext(Box::new(std::fmt::Error))).is_none());
}

#[cfg(windows)]
#[test]
fn transient_windows_reader_releases_before_bounded_publication_finishes() {
    use std::{os::windows::fs::OpenOptionsExt, time::Duration};
    let directory = FixtureDirectory::new("transient-reader");
    let path = directory.checkpoint();
    fs::write(&path, OLD_GENERATION).unwrap();
    let reader = fs::OpenOptions::new()
        .read(true)
        .share_mode(1 | 2)
        .open(&path)
        .unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        drop(reader);
    });

    let publication = write_json(&path, &serde_json::json!({ "tick": 2 }));
    release.join().unwrap();
    publication.unwrap();

    assert_eq!(fs::read(&path).unwrap(), NEW_GENERATION);
    assert!(!path.with_extension("pending").exists());
}

#[cfg(windows)]
#[test]
fn persistent_windows_denial_is_bounded_preserves_old_and_recovers() {
    use std::{
        os::windows::fs::OpenOptionsExt,
        time::{Duration, Instant},
    };
    let directory = FixtureDirectory::new("persistent-reader");
    let path = directory.checkpoint();
    let pending = path.with_extension("pending");
    fs::write(&path, OLD_GENERATION).unwrap();
    fs::write(&pending, NEW_GENERATION).unwrap();
    let reader = fs::OpenOptions::new()
        .read(true)
        .share_mode(1 | 2)
        .open(&path)
        .unwrap();
    let original_error = fs::rename(&pending, &path).unwrap_err();
    assert!(matches!(original_error.raw_os_error(), Some(5 | 32 | 33)));

    let started = Instant::now();
    let error = write_json(&path, &serde_json::json!({ "tick": 2 })).unwrap_err();
    let elapsed = started.elapsed();

    assert!(elapsed >= Duration::from_millis(100));
    assert!(
        elapsed < Duration::from_secs(2),
        "publication took {elapsed:?}"
    );
    assert_eq!(
        error.to_string(),
        format!(
            "Cannot publish checkpoint {}: {original_error}",
            path.display()
        )
    );
    assert_eq!(fs::read(&path).unwrap(), OLD_GENERATION);
    assert_eq!(fs::read(&pending).unwrap(), NEW_GENERATION);
    eprintln!(
        "persistent Windows denial elapsed: {elapsed:?}; raw OS error: {:?}; kind: {:?}",
        original_error.raw_os_error(),
        original_error.kind()
    );
    let source = error
        .source()
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .expect("publication error must retain its original I/O source");
    assert_eq!(source.kind(), original_error.kind());
    assert_eq!(source.raw_os_error(), original_error.raw_os_error());
    assert!(std::ptr::eq(io_error(error.as_ref()).unwrap(), source));

    drop(reader);
    write_json(&path, &serde_json::json!({ "tick": 3 })).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"{\n  \"tick\": 3\n}");
    assert!(!pending.exists());
}
