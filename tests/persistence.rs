use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use turbofish_deluxe::{cli, sim::AdventureState};

fn temporary_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "turbofish-{name}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn atomic_save_replaces_complete_snapshot_and_retains_seeded_state() {
    let root = temporary_root("save");
    let path = root.join("adventure.json");
    let mut state = AdventureState::new_adventure(42);
    cli::write_json(
        &path,
        &cli::ProjectSave {
            format_version: 1,
            state: state.clone(),
        },
    )
    .unwrap();
    for _ in 0..20 {
        state.tick();
    }
    cli::write_json(
        &path,
        &cli::ProjectSave {
            format_version: 1,
            state: state.clone(),
        },
    )
    .unwrap();
    let loaded: cli::ProjectSave = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(loaded.format_version, 1);
    assert_eq!(loaded.state.tick, 20);
    assert_eq!(loaded.state.balance, 200);
    assert_eq!(loaded.state.fish.len(), 2);
    let mut resumed = loaded.state;
    assert_eq!(resumed.tick(), state.tick());
    assert_eq!(
        serde_json::to_value(resumed).unwrap(),
        serde_json::to_value(state).unwrap()
    );
    assert!(!path.with_extension("pending").exists());
    fs::remove_file(path).unwrap();
    fs::remove_dir(root).unwrap();
}

#[cfg(windows)]
#[test]
fn denied_atomic_replace_preserves_previous_complete_snapshot() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = temporary_root("read-lock");
    let path = root.join("state.local.json");
    cli::write_json(&path, &serde_json::json!({"tick":1})).unwrap();
    let reader = fs::OpenOptions::new()
        .read(true)
        .share_mode(1 | 2)
        .open(&path)
        .unwrap();
    assert!(cli::write_json(&path, &serde_json::json!({"tick":2})).is_err());
    let previous: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(previous["tick"], 1);
    drop(reader);
    cli::write_json(&path, &serde_json::json!({"tick":3})).unwrap();
    let recovered: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(recovered["tick"], 3);
    fs::remove_file(path).unwrap();
    fs::remove_dir(root).unwrap();
}
