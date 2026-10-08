//! Read-only adapter for a player's owned installation. Decoded pixels and samples
//! are transient memory; no converted game assets are written to disk.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub enum AssetError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Manifest(String),
    Missing(String),
    Invalid(String),
    Decode {
        path: PathBuf,
        source: image::ImageError,
    },
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Manifest(message) => write!(formatter, "invalid resource manifest: {message}"),
            Self::Missing(message) => write!(formatter, "missing game asset: {message}"),
            Self::Invalid(message) => write!(formatter, "invalid game asset: {message}"),
            Self::Decode { path, source } => write!(formatter, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for AssetError {}

#[derive(Clone, Debug)]
pub struct ImageEntry {
    pub id: String,
    pub path: PathBuf,
    pub cols: u32,
    pub rows: u32,
    alpha: Option<AlphaSource>,
}

#[derive(Clone, Debug)]
enum AlphaSource {
    Full(PathBuf),
    Grid(PathBuf),
}

#[derive(Clone, Debug)]
struct ResourceEntry {
    path: PathBuf,
}

struct ParsedManifest {
    images: HashMap<String, ImageEntry>,
    sounds: HashMap<String, ResourceEntry>,
}

#[derive(Debug)]
pub struct GameAssets {
    root: PathBuf,
    images: HashMap<String, ImageEntry>,
    sounds: HashMap<String, ResourceEntry>,
}

#[derive(Debug)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    /// Straight-alpha RGBA8, row-major.
    pub pixels: Vec<u8>,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Debug)]
pub enum SoundData {
    Ogg(Vec<u8>),
    Pcm16 { sample_rate: u32, samples: Vec<i16> },
}

impl GameAssets {
    pub fn open(game_dir: impl AsRef<Path>) -> Result<Self, AssetError> {
        let supplied = game_dir.as_ref();
        let root = fs::canonicalize(supplied).map_err(|source| AssetError::Io {
            path: supplied.to_owned(),
            source,
        })?;
        if !root.is_dir() {
            return Err(AssetError::Invalid(format!(
                "{} is not a directory",
                root.display()
            )));
        }
        let manifest_path = root.join("properties/resources.xml");
        let resolved = fs::canonicalize(&manifest_path).map_err(|source| AssetError::Io {
            path: manifest_path.clone(),
            source,
        })?;
        if !resolved.starts_with(&root) {
            return Err(AssetError::Invalid(format!(
                "resource manifest escapes game directory: {}",
                resolved.display()
            )));
        }
        let manifest = fs::read_to_string(&manifest_path).map_err(|source| AssetError::Io {
            path: manifest_path,
            source,
        })?;
        let ParsedManifest { images, sounds } = parse_manifest(&manifest)?;
        Ok(Self {
            root,
            images,
            sounds,
        })
    }

    pub fn image_entry(&self, id: &str) -> Option<&ImageEntry> {
        self.images.get(id)
    }

    pub fn image_ids(&self) -> impl Iterator<Item = &str> {
        self.images.keys().map(String::as_str)
    }

    pub fn sound_ids(&self) -> impl Iterator<Item = &str> {
        self.sounds.keys().map(String::as_str)
    }

    pub fn load_image(&self, id_or_stem: &str) -> Result<DecodedImage, AssetError> {
        let (relative, cols, rows, alpha) = match self.images.get(id_or_stem) {
            Some(entry) => (
                entry.path.clone(),
                entry.cols,
                entry.rows,
                entry.alpha.as_ref(),
            ),
            None => (fallback_path("images", id_or_stem)?, 1, 1, None),
        };
        let color = self.find_extension(&relative, &["jpg", "png", "gif"])?;
        let mask_path = if let Some(source) = alpha {
            let path = match source {
                AlphaSource::Full(path) | AlphaSource::Grid(path) => path,
            };
            Some(
                self.find_extension(path, &["jpg", "png", "gif"])?
                    .ok_or_else(|| {
                        AssetError::Missing(format!(
                            "declared alpha image {} for {id_or_stem}",
                            path.display()
                        ))
                    })?,
            )
        } else {
            let mut found = None;
            for stem in companion_stems(&relative) {
                if let Some(path) = self.find_extension(&stem, &["jpg", "png", "gif"])? {
                    found = Some(path);
                    break;
                }
            }
            found
        };
        if color.is_none() && mask_path.is_none() {
            return Err(AssetError::Missing(format!(
                "image {id_or_stem} ({})",
                relative.display()
            )));
        }
        if color.is_none() && alpha.is_some() {
            return Err(AssetError::Missing(format!(
                "declared color image {id_or_stem} ({})",
                relative.display()
            )));
        }
        let mut rgba = if let Some(path) = color {
            image::open(&path)
                .map_err(|source| AssetError::Decode { path, source })?
                .to_rgba8()
        } else {
            let path = mask_path.as_ref().expect("at least one image exists");
            let mask = image::open(path).map_err(|source| AssetError::Decode {
                path: path.clone(),
                source,
            })?;
            image::RgbaImage::from_pixel(
                mask.width(),
                mask.height(),
                image::Rgba([255, 255, 255, 255]),
            )
        };
        if let Some(path) = mask_path {
            let mask = image::open(&path)
                .map_err(|source| AssetError::Decode { path, source })?
                .to_rgb8();
            let (width, height) = rgba.dimensions();
            let expected = if matches!(alpha, Some(AlphaSource::Grid(_))) {
                if width % cols != 0 || height % rows != 0 {
                    return Err(AssetError::Invalid(format!(
                        "image {id_or_stem}: grid does not divide source"
                    )));
                }
                (width / cols, height / rows)
            } else {
                (width, height)
            };
            if mask.dimensions() != expected {
                if alpha.is_some() {
                    return Err(AssetError::Invalid(format!(
                        "mask dimensions {}x{} disagree with expected {}x{} for {id_or_stem}",
                        mask.width(),
                        mask.height(),
                        expected.0,
                        expected.1
                    )));
                }
                // The stock install includes unrelated companions (for example
                // EDITBOX). Automatic masks apply only when dimensions match.
            } else {
                for y in 0..height {
                    for x in 0..width {
                        rgba.get_pixel_mut(x, y).0[3] =
                            mask.get_pixel(x % expected.0, y % expected.1).0[2];
                    }
                }
            }
        }
        let (width, height) = rgba.dimensions();
        if cols == 0 || rows == 0 || width % cols != 0 || height % rows != 0 {
            return Err(AssetError::Invalid(format!(
                "image {id_or_stem}: {}x{} cannot be divided into {cols}x{rows} cells",
                width, height
            )));
        }
        Ok(DecodedImage {
            width,
            height,
            pixels: rgba.into_raw(),
            cols,
            rows,
        })
    }

    pub fn load_sound(&self, id_or_stem: &str) -> Result<SoundData, AssetError> {
        let relative = match self.sounds.get(id_or_stem) {
            Some(entry) => entry.path.clone(),
            None => fallback_path("sounds", id_or_stem)?,
        };
        let path = self
            .find_extension(&relative, &["ogg", "au"])?
            .ok_or_else(|| {
                AssetError::Missing(format!("sound {id_or_stem} ({})", relative.display()))
            })?;
        let bytes = fs::read(&path).map_err(|source| AssetError::Io {
            path: path.clone(),
            source,
        })?;
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("ogg") => Ok(SoundData::Ogg(bytes)),
            Some(extension) if extension.eq_ignore_ascii_case("au") => decode_au(&bytes),
            _ => Err(AssetError::Invalid(format!(
                "unsupported sound {}",
                path.display()
            ))),
        }
    }

    fn find_extension(
        &self,
        relative: &Path,
        extensions: &[&str],
    ) -> Result<Option<PathBuf>, AssetError> {
        let mut choices = Vec::with_capacity(extensions.len() + 1);
        if relative.extension().is_some() {
            choices.push(relative.to_owned());
        } else {
            choices.extend(
                extensions
                    .iter()
                    .map(|extension| relative.with_extension(extension)),
            );
        }
        for choice in choices {
            let candidate = self.root.join(choice);
            if candidate.is_file() {
                let resolved = fs::canonicalize(&candidate).map_err(|source| AssetError::Io {
                    path: candidate,
                    source,
                })?;
                if !resolved.starts_with(&self.root) {
                    return Err(AssetError::Invalid(format!(
                        "asset path escapes game directory: {}",
                        resolved.display()
                    )));
                }
                return Ok(Some(resolved));
            }
        }
        Ok(None)
    }
}

fn companion_stems(relative: &Path) -> Vec<PathBuf> {
    let Some(name) = relative.file_stem().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    if name.starts_with('_') || name.ends_with('_') {
        return Vec::new();
    }
    let mut result = Vec::new();
    for companion in [format!("_{name}"), format!("{name}_")] {
        let mut path = relative.with_file_name(companion);
        path.set_extension("");
        result.push(path);
    }
    result
}

fn safe_relative(value: &str) -> Result<PathBuf, AssetError> {
    let normalized = value.replace('\\', "/");
    let path = PathBuf::from(normalized);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(AssetError::Invalid(format!(
            "unsafe relative asset path {value:?}"
        )));
    }
    Ok(path)
}

fn fallback_path(directory: &str, stem: &str) -> Result<PathBuf, AssetError> {
    let relative = safe_relative(stem)?;
    if relative.components().count() == 1 {
        Ok(Path::new(directory).join(relative))
    } else {
        Ok(relative)
    }
}

fn parse_manifest(manifest: &str) -> Result<ParsedManifest, AssetError> {
    let mut images = HashMap::new();
    let mut sounds = HashMap::new();
    let mut cursor = 0;
    let mut in_root = false;
    let mut in_group = false;
    let mut defaults_path = String::new();
    let mut id_prefix = String::new();
    let bytes = manifest.as_bytes();
    while let Some(start_offset) = manifest[cursor..].find('<') {
        let start = cursor + start_offset;
        if manifest[start..].starts_with("<!--") {
            let end = manifest[start + 4..]
                .find("-->")
                .ok_or_else(|| AssetError::Manifest("unclosed comment".into()))?;
            cursor = start + 4 + end + 3;
            continue;
        }
        let mut end = start + 1;
        let mut quote = None;
        while end < bytes.len() {
            let current = bytes[end];
            if let Some(delimiter) = quote {
                if current == delimiter {
                    quote = None;
                }
            } else if current == b'\'' || current == b'"' {
                quote = Some(current);
            } else if current == b'>' {
                break;
            }
            end += 1;
        }
        if end == bytes.len() {
            return Err(AssetError::Manifest("unclosed tag".into()));
        }
        let tag = manifest[start + 1..end].trim();
        cursor = end + 1;
        if tag.starts_with('?') || tag.starts_with('!') {
            continue;
        }
        if let Some(closing) = tag.strip_prefix('/') {
            match closing.trim() {
                "Resources" if in_group => in_group = false,
                "ResourceManifest" if in_root && !in_group => in_root = false,
                other => {
                    return Err(AssetError::Manifest(format!(
                        "unexpected closing tag {other}"
                    )));
                }
            }
            continue;
        }
        let tag = tag.trim_end_matches('/').trim_end();
        let name_end = tag.find(char::is_whitespace).unwrap_or(tag.len());
        let name = &tag[..name_end];
        let attrs = parse_attributes(&tag[name_end..])?;
        match name {
            "ResourceManifest" if !in_root => in_root = true,
            "Resources" if in_root && !in_group => {
                in_group = true;
                defaults_path.clear();
                id_prefix.clear();
            }
            "SetDefaults" if in_group => {
                if let Some(value) = attrs.get("path") {
                    defaults_path = value.clone();
                }
                if let Some(value) = attrs.get("idprefix") {
                    id_prefix = value.clone();
                }
            }
            "Image" | "Sound" | "Font" if in_group => {
                let local_id = required_attr(&attrs, "id", name)?;
                let local_path = required_attr(&attrs, "path", name)?;
                let relative = if defaults_path.is_empty() {
                    local_path.to_owned()
                } else {
                    format!("{defaults_path}/{local_path}")
                };
                let path = safe_relative(&relative)
                    .map_err(|error| AssetError::Manifest(error.to_string()))?;
                let id = format!("{id_prefix}{local_id}");
                if name == "Image" {
                    let cols = positive_grid(&attrs, "cols")?;
                    let rows = positive_grid(&attrs, "rows")?;
                    let alpha = match (attrs.get("alphaimage"), attrs.get("alphagrid")) {
                        (Some(_), Some(_)) => {
                            return Err(AssetError::Manifest(format!(
                                "image {id} declares two alpha sources"
                            )));
                        }
                        (Some(value), None) => {
                            Some(AlphaSource::Full(defaulted_path(&defaults_path, value)?))
                        }
                        (None, Some(value)) => {
                            Some(AlphaSource::Grid(defaulted_path(&defaults_path, value)?))
                        }
                        (None, None) => None,
                    };
                    let entry = ImageEntry {
                        id: id.clone(),
                        path,
                        cols,
                        rows,
                        alpha,
                    };
                    if images.insert(id.clone(), entry).is_some() {
                        return Err(AssetError::Manifest(format!("duplicate image {id}")));
                    }
                } else if name == "Sound"
                    && sounds.insert(id.clone(), ResourceEntry { path }).is_some()
                {
                    return Err(AssetError::Manifest(format!("duplicate sound {id}")));
                }
            }
            _ => return Err(AssetError::Manifest(format!("unexpected tag {name}"))),
        }
    }
    if in_root || in_group || images.is_empty() {
        return Err(AssetError::Manifest(
            "unclosed or empty resource manifest".into(),
        ));
    }
    Ok(ParsedManifest { images, sounds })
}

fn defaulted_path(defaults: &str, path: &str) -> Result<PathBuf, AssetError> {
    let relative = if defaults.is_empty() {
        path.to_owned()
    } else {
        format!("{defaults}/{path}")
    };
    safe_relative(&relative).map_err(|error| AssetError::Manifest(error.to_string()))
}

fn required_attr<'a>(
    attrs: &'a HashMap<String, String>,
    key: &str,
    tag: &str,
) -> Result<&'a str, AssetError> {
    attrs
        .get(key)
        .filter(|value| !value.is_empty())
        .map(String::as_str)
        .ok_or_else(|| AssetError::Manifest(format!("{tag} missing {key}")))
}

fn positive_grid(attrs: &HashMap<String, String>, key: &str) -> Result<u32, AssetError> {
    match attrs.get(key) {
        None => Ok(1),
        Some(value) => value
            .parse()
            .ok()
            .filter(|number| *number > 0)
            .ok_or_else(|| AssetError::Manifest(format!("invalid {key}={value:?}"))),
    }
}

fn parse_attributes(input: &str) -> Result<HashMap<String, String>, AssetError> {
    let mut values = HashMap::new();
    let bytes = input.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        while at < bytes.len() && bytes[at].is_ascii_whitespace() {
            at += 1;
        }
        if at == bytes.len() {
            break;
        }
        let key_start = at;
        while at < bytes.len()
            && (bytes[at].is_ascii_alphanumeric() || matches!(bytes[at], b'_' | b'-'))
        {
            at += 1;
        }
        if at == key_start {
            return Err(AssetError::Manifest(format!(
                "bad attribute near {:?}",
                &input[at..]
            )));
        }
        let key = &input[key_start..at];
        while at < bytes.len() && bytes[at].is_ascii_whitespace() {
            at += 1;
        }
        let value = if at < bytes.len() && bytes[at] == b'=' {
            at += 1;
            while at < bytes.len() && bytes[at].is_ascii_whitespace() {
                at += 1;
            }
            if at == bytes.len() {
                return Err(AssetError::Manifest(format!("missing value for {key}")));
            }
            if bytes[at] == b'"' || bytes[at] == b'\'' {
                let delimiter = bytes[at];
                at += 1;
                let value_start = at;
                while at < bytes.len() && bytes[at] != delimiter {
                    at += 1;
                }
                if at == bytes.len() {
                    return Err(AssetError::Manifest(format!("unclosed value for {key}")));
                }
                let value = &input[value_start..at];
                at += 1;
                value
            } else {
                let value_start = at;
                while at < bytes.len() && !bytes[at].is_ascii_whitespace() {
                    at += 1;
                }
                &input[value_start..at]
            }
        } else {
            ""
        }; // The original manifest uses valueless ddsurface flags.
        if values.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(AssetError::Manifest(format!("duplicate attribute {key}")));
        }
    }
    Ok(values)
}

fn decode_au(bytes: &[u8]) -> Result<SoundData, AssetError> {
    if bytes.len() < 24 || &bytes[..4] != b".snd" {
        return Err(AssetError::Invalid(
            "AU header is missing or truncated".into(),
        ));
    }
    let number = |start: usize| {
        u32::from_be_bytes(
            bytes[start..start + 4]
                .try_into()
                .expect("header length checked"),
        )
    };
    let offset = number(4) as usize;
    let length = number(8);
    let encoding = number(12);
    let sample_rate = number(16);
    let channels = number(20);
    if offset < 24 || offset > bytes.len() || encoding != 1 || sample_rate == 0 || channels != 1 {
        return Err(AssetError::Invalid(format!(
            "unsupported AU header: offset={offset}, encoding={encoding}, rate={sample_rate}, channels={channels}"
        )));
    }
    let available = bytes.len() - offset;
    let count = if length == u32::MAX {
        available
    } else {
        usize::try_from(length).unwrap_or(usize::MAX)
    };
    if count > available {
        return Err(AssetError::Invalid("truncated AU samples".into()));
    }
    let samples = bytes[offset..offset + count]
        .iter()
        .map(|&byte| {
            let complemented = !byte;
            let magnitude = (((u32::from(complemented & 0x0f) << 3) + 0x84)
                << ((complemented >> 4) & 0x07)) as i32
                - 0x84;
            let signed = if complemented & 0x80 != 0 {
                -magnitude
            } else {
                magnitude
            };
            signed.clamp(i16::MIN as i32, i16::MAX as i32) as i16
        })
        .collect();
    Ok(SoundData::Pcm16 {
        sample_rate,
        samples,
    })
}
