//! Owned installation discovery and identity. This module never writes game data.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallIdentity {
    pub executable: String,
    pub executable_sha256: String,
    pub executable_bytes: u64,
    pub pe_machine: u16,
    pub asset_file_count: usize,
    pub inventory_sha256: String,
    pub steam_build: Option<String>,
}

pub fn discover(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return validate(path);
    }
    if let Some(path) = env::var_os("TURBOFISH_GAME_DIR") {
        return validate(Path::new(&path));
    }
    let mut steam_roots = Vec::new();
    for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(base) = env::var_os(variable) {
            steam_roots.push(PathBuf::from(base).join("Steam"));
        }
    }
    for root in [
        PathBuf::from("C:/Steam"),
        PathBuf::from("C:/SteamLibrary"),
        PathBuf::from("D:/SteamLibrary"),
    ] {
        if !steam_roots.contains(&root) {
            steam_roots.push(root);
        }
    }
    let mut libraries = steam_roots.clone();
    for root in steam_roots {
        for manifest in [
            root.join("steamapps/libraryfolders.vdf"),
            root.join("config/libraryfolders.vdf"),
        ] {
            if let Ok(text) = fs::read_to_string(manifest) {
                for value in values_for_key(&text, "path") {
                    let path = PathBuf::from(value);
                    if !libraries.contains(&path) {
                        libraries.push(path);
                    }
                }
            }
        }
    }
    for library in libraries {
        let steamapps = library.join("steamapps");
        let manifest = steamapps.join("appmanifest_3320.acf");
        if let Ok(text) = fs::read_to_string(manifest)
            && let Some(directory) = values_for_key(&text, "installdir").into_iter().next()
            && let Ok(path) = validate(&steamapps.join("common").join(directory))
        {
            return Ok(path);
        }
        if let Ok(path) = validate(&steamapps.join("common/Insaniquarium Deluxe")) {
            return Ok(path);
        }
    }
    Err("No owned Insaniquarium installation found. Use --game-dir <directory> or TURBOFISH_GAME_DIR.".into())
}

fn validate(path: &Path) -> Result<PathBuf> {
    let root = path
        .canonicalize()
        .map_err(|error| format!("Game directory {}: {error}", path.display()))?;
    if !root.join("properties/resources.xml").is_file()
        || !root.join("images/aquarium1.jpg").is_file()
    {
        return Err(format!(
            "{} is missing Insaniquarium resources.xml or aquarium1.jpg",
            root.display()
        )
        .into());
    }
    if !root.join("InsaniquariumDeluxe.exe").is_file() && !root.join("Insaniquarium.exe").is_file()
    {
        return Err(format!("{} has no recognized reference executable", root.display()).into());
    }
    Ok(root)
}

pub fn identify(root: &Path) -> Result<InstallIdentity> {
    let executable = if root.join("InsaniquariumDeluxe.exe").is_file() {
        "InsaniquariumDeluxe.exe"
    } else {
        "Insaniquarium.exe"
    };
    let bytes = fs::read(root.join(executable))?;
    let pe_machine = pe_machine(&bytes)?;
    let mut files = Vec::new();
    inventory(root, root, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut inventory_digest = Sha256::new();
    for (path, bytes) in &files {
        inventory_digest.update(path.as_bytes());
        inventory_digest.update([0]);
        inventory_digest.update((bytes.len() as u64).to_le_bytes());
        inventory_digest.update(Sha256::digest(bytes));
    }
    let steam_build = root
        .parent()
        .and_then(Path::parent)
        .and_then(|steamapps| fs::read_to_string(steamapps.join("appmanifest_3320.acf")).ok())
        .and_then(|text| values_for_key(&text, "buildid").into_iter().next());
    Ok(InstallIdentity {
        executable: executable.into(),
        executable_sha256: sha256(&bytes),
        executable_bytes: bytes.len() as u64,
        pe_machine,
        asset_file_count: files.len(),
        inventory_sha256: hex(&inventory_digest.finalize()),
        steam_build,
    })
}

fn inventory(root: &Path, directory: &Path, files: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let resolved = path.canonicalize()?;
        if !resolved.starts_with(root) {
            return Err(format!(
                "Game inventory link escapes installation: {}",
                path.display()
            )
            .into());
        }
        if entry.file_type()?.is_dir() {
            inventory(root, &path, files)?;
        } else if entry.file_type()?.is_file() {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, fs::read(path)?));
        }
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn pe_machine(bytes: &[u8]) -> Result<u16> {
    if bytes.get(..2) != Some(b"MZ") {
        return Err("Reference executable lacks DOS header".into());
    }
    let offset_bytes: [u8; 4] = bytes
        .get(0x3c..0x40)
        .ok_or("Truncated DOS header")?
        .try_into()?;
    let offset = u32::from_le_bytes(offset_bytes) as usize;
    if bytes.get(offset..offset.saturating_add(4)) != Some(b"PE\0\0") {
        return Err("Reference executable lacks PE signature".into());
    }
    let machine: [u8; 2] = bytes
        .get(offset + 4..offset + 6)
        .ok_or("Truncated PE header")?
        .try_into()?;
    Ok(u16::from_le_bytes(machine))
}

/// Small read-only Valve KeyValues tokenizer. Quoted values preserve Windows paths.
pub fn values_for_key(text: &str, key: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = text.trim_start_matches('\u{feff}').chars().peekable();
    while let Some(character) = chars.next() {
        if character == '/' && chars.peek() == Some(&'/') {
            for next in chars.by_ref() {
                if next == '\n' {
                    break;
                }
            }
        } else if character == '"' {
            let mut token = String::new();
            while let Some(next) = chars.next() {
                if next == '"' {
                    break;
                }
                if next == '\\' && matches!(chars.peek(), Some('\\' | '"')) {
                    token.push(chars.next().expect("peeked character"));
                } else {
                    token.push(next);
                }
            }
            tokens.push(token);
        } else if character == '{' || character == '}' {
            tokens.push(character.to_string());
        }
    }
    tokens
        .windows(2)
        .filter(|pair| pair[0].eq_ignore_ascii_case(key) && pair[1] != "{")
        .map(|pair| pair[1].clone())
        .collect()
}

/// Resolve a write destination using its nearest existing ancestor, including junctions.
/// Project evidence/saves may never be placed inside the owned installation.
pub fn project_destination(path: &Path, game_root: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut ancestor = absolute.as_path();
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or("No existing destination ancestor")?;
    }
    let canonical_ancestor = ancestor.canonicalize()?;
    if canonical_ancestor.starts_with(game_root) {
        return Err("Project writes cannot target the original game installation".into());
    }
    if absolute
        .components()
        .any(|part| matches!(part, std::path::Component::ParentDir))
    {
        return Err("Write destination must not contain parent traversal".into());
    }
    Ok(absolute)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyvalues_preserves_escaped_paths_and_ignores_comments() {
        let text = r#""libraryfolders" { "0" { "path" "C:\\Program Files (x86)\\Steam" } // "path" "wrong"
        "1" { "path" "D:\\SteamLibrary" } }"#;
        assert_eq!(
            values_for_key(text, "PATH"),
            ["C:\\Program Files (x86)\\Steam", "D:\\SteamLibrary"]
        );
    }
    #[test]
    fn malformed_reference_is_rejected() {
        assert!(pe_machine(b"MZ").is_err());
        assert!(pe_machine(&[0; 100]).is_err());
        let mut bytes = vec![0; 100];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&80_u32.to_le_bytes());
        bytes[80..84].copy_from_slice(b"PE\0\0");
        bytes[84..86].copy_from_slice(&0x14c_u16.to_le_bytes());
        assert_eq!(pe_machine(&bytes).unwrap(), 0x14c);
    }
}
