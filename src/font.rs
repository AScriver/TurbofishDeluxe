//! Bounded reader for the owned game's declarative bitmap-font data scripts.
//! The font atlas remains an owned-install image loaded by `GameAssets`.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug)]
pub enum FontError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Invalid(String),
}

impl fmt::Display for FontError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Invalid(message) => write!(formatter, "invalid bitmap font: {message}"),
        }
    }
}

impl std::error::Error for FontError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlyphRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Debug, Default)]
pub struct Glyph {
    pub advance: i32,
    pub rect: Option<GlyphRect>,
    pub offset_x: i32,
    pub offset_y: i32,
}

#[derive(Debug)]
pub struct FontLayer {
    pub name: String,
    /// Relative to the game root; load through `GameAssets::load_image`.
    pub image_path: PathBuf,
    pub ascent: i32,
    pub ascent_padding: i32,
    pub line_spacing_offset: i32,
    pub point_size: i32,
    pub spacing: i32,
    pub glyphs: HashMap<char, Glyph>,
    pub kerning: HashMap<(char, char), i32>,
}

impl FontLayer {
    fn new(name: String) -> Self {
        Self {
            name,
            image_path: PathBuf::new(),
            ascent: 0,
            ascent_padding: 0,
            line_spacing_offset: 0,
            point_size: 0,
            spacing: 0,
            glyphs: HashMap::new(),
            kerning: HashMap::new(),
        }
    }

    pub fn validate_atlas(&self, width: u32, height: u32) -> Result<(), FontError> {
        for (character, glyph) in &self.glyphs {
            if let Some(rect) = glyph.rect {
                let right = rect
                    .x
                    .checked_add(rect.width)
                    .ok_or_else(|| FontError::Invalid("glyph rectangle overflow".into()))?;
                let bottom = rect
                    .y
                    .checked_add(rect.height)
                    .ok_or_else(|| FontError::Invalid("glyph rectangle overflow".into()))?;
                if rect.x < 0
                    || rect.y < 0
                    || rect.width < 0
                    || rect.height < 0
                    || i64::from(right) > i64::from(width)
                    || i64::from(bottom) > i64::from(height)
                {
                    return Err(FontError::Invalid(format!(
                        "glyph {character:?} lies outside atlas {}x{}",
                        width, height
                    )));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BitmapFont {
    pub default_point_size: i32,
    pub layers: Vec<FontLayer>,
}

impl BitmapFont {
    pub fn load(game_dir: impl AsRef<Path>, script_stem: &str) -> Result<Self, FontError> {
        let root = fs::canonicalize(game_dir.as_ref()).map_err(|source| FontError::Io {
            path: game_dir.as_ref().to_owned(),
            source,
        })?;
        if !root.is_dir() {
            return Err(FontError::Invalid("game root is not a directory".into()));
        }
        let script_stem = script_stem.strip_suffix(".txt").unwrap_or(script_stem);
        let name = safe_relative(script_stem)?;
        if name.components().count() != 1 {
            return Err(FontError::Invalid(
                "font name must be a single file stem".into(),
            ));
        }
        let path = root.join("data").join(name).with_extension("txt");
        let resolved = fs::canonicalize(&path).map_err(|source| FontError::Io {
            path: path.clone(),
            source,
        })?;
        if !resolved.starts_with(&root) {
            return Err(FontError::Invalid("font script escapes game root".into()));
        }
        let bytes = fs::read(&resolved).map_err(|source| FontError::Io {
            path: resolved,
            source,
        })?;
        let script = decode_windows_1252(&bytes)?;
        Self::parse(&script)
    }

    pub fn parse(script: &str) -> Result<Self, FontError> {
        let statements = statements(&lex(script)?)?;
        let mut definitions = HashMap::new();
        let mut layers: Vec<FontLayer> = Vec::new();
        let mut default_point_size = None;
        for (command, args) in statements {
            if command == "Define" {
                expect_count(&command, &args, 2)?;
                let name = atom(&args[0])?.to_owned();
                if definitions.insert(name.clone(), args[1].clone()).is_some() {
                    return Err(FontError::Invalid(format!("duplicate definition {name}")));
                }
                continue;
            }
            if command == "CreateLayer" {
                expect_count(&command, &args, 1)?;
                let name = atom(&args[0])?.to_owned();
                if layers.iter().any(|layer| layer.name == name) {
                    return Err(FontError::Invalid(format!("duplicate layer {name}")));
                }
                layers.push(FontLayer::new(name));
                continue;
            }
            if command == "SetDefaultPointSize" {
                expect_count(&command, &args, 1)?;
                default_point_size = Some(integer(&args[0])?);
                continue;
            }
            if !command.starts_with("Layer") {
                return Err(FontError::Invalid(format!("unsupported command {command}")));
            }
            let expected = match command.as_str() {
                "LayerSetImage"
                | "LayerSetAscent"
                | "LayerSetAscentPadding"
                | "LayerSetLineSpacingOffset"
                | "LayerSetPointSize"
                | "LayerSetSpacing" => 2,
                "LayerSetCharWidths"
                | "LayerSetImageMap"
                | "LayerSetCharOffsets"
                | "LayerSetKerningPairs" => 3,
                _ => return Err(FontError::Invalid(format!("unsupported command {command}"))),
            };
            expect_count(&command, &args, expected)?;
            let layer_name = atom(&args[0])?;
            let layer = layers
                .iter_mut()
                .find(|layer| layer.name == layer_name)
                .ok_or_else(|| {
                    FontError::Invalid(format!("{command} refers to missing layer {layer_name}"))
                })?;
            match command.as_str() {
                "LayerSetImage" => {
                    let name = safe_relative(atom(&args[1])?)?;
                    if name.components().count() != 1 {
                        return Err(FontError::Invalid(
                            "font atlas name must be a single stem".into(),
                        ));
                    }
                    layer.image_path = Path::new("data").join(name);
                }
                "LayerSetAscent" => layer.ascent = integer(&args[1])?,
                "LayerSetAscentPadding" => layer.ascent_padding = integer(&args[1])?,
                "LayerSetLineSpacingOffset" => layer.line_spacing_offset = integer(&args[1])?,
                "LayerSetPointSize" => layer.point_size = integer(&args[1])?,
                "LayerSetSpacing" => layer.spacing = integer(&args[1])?,
                "LayerSetCharWidths" => {
                    let chars = list(resolve(&args[1], &definitions)?)?;
                    let widths = list(resolve(&args[2], &definitions)?)?;
                    check_lengths(&command, &chars, &widths)?;
                    for (character, width) in chars.iter().zip(&widths) {
                        layer
                            .glyphs
                            .entry(single_char(character)?)
                            .or_default()
                            .advance = integer(width)?;
                    }
                }
                "LayerSetImageMap" => {
                    let chars = list(resolve(&args[1], &definitions)?)?;
                    let rectangles = list(resolve(&args[2], &definitions)?)?;
                    check_lengths(&command, &chars, &rectangles)?;
                    for (character, rectangle) in chars.iter().zip(&rectangles) {
                        let numbers = list(rectangle.clone())?;
                        if numbers.len() != 4 {
                            return Err(FontError::Invalid(
                                "glyph rectangle needs four integers".into(),
                            ));
                        }
                        let rect = GlyphRect {
                            x: integer(&numbers[0])?,
                            y: integer(&numbers[1])?,
                            width: integer(&numbers[2])?,
                            height: integer(&numbers[3])?,
                        };
                        if rect.x < 0 || rect.y < 0 || rect.width < 0 || rect.height < 0 {
                            return Err(FontError::Invalid("negative glyph rectangle".into()));
                        }
                        layer
                            .glyphs
                            .entry(single_char(character)?)
                            .or_default()
                            .rect = Some(rect);
                    }
                }
                "LayerSetCharOffsets" => {
                    let chars = list(resolve(&args[1], &definitions)?)?;
                    let offsets = list(resolve(&args[2], &definitions)?)?;
                    check_lengths(&command, &chars, &offsets)?;
                    for (character, offset) in chars.iter().zip(&offsets) {
                        let numbers = list(offset.clone())?;
                        if numbers.len() != 2 {
                            return Err(FontError::Invalid(
                                "glyph offset needs two integers".into(),
                            ));
                        }
                        let glyph = layer.glyphs.entry(single_char(character)?).or_default();
                        glyph.offset_x = integer(&numbers[0])?;
                        glyph.offset_y = integer(&numbers[1])?;
                    }
                }
                "LayerSetKerningPairs" => {
                    let pairs = list(resolve(&args[1], &definitions)?)?;
                    let values = list(resolve(&args[2], &definitions)?)?;
                    check_lengths(&command, &pairs, &values)?;
                    for (pair, value) in pairs.iter().zip(&values) {
                        let characters: Vec<_> = atom(pair)?.chars().collect();
                        if characters.len() != 2 {
                            return Err(FontError::Invalid(
                                "kerning pair needs two characters".into(),
                            ));
                        }
                        layer
                            .kerning
                            .insert((characters[0], characters[1]), integer(value)?);
                    }
                }
                _ => unreachable!("command checked above"),
            }
        }
        let default_point_size = default_point_size
            .ok_or_else(|| FontError::Invalid("missing default point size".into()))?;
        if default_point_size <= 0
            || layers.is_empty()
            || layers.iter().any(|layer| {
                layer.image_path.as_os_str().is_empty()
                    || layer.point_size <= 0
                    || layer.glyphs.is_empty()
            })
        {
            return Err(FontError::Invalid(
                "incomplete font layer or point size".into(),
            ));
        }
        Ok(Self {
            default_point_size,
            layers,
        })
    }

    /// Unscaled width at the script's declared point size, using the first layer.
    pub fn measure_width(&self, text: &str) -> Result<i32, FontError> {
        let layer = self
            .layers
            .first()
            .ok_or_else(|| FontError::Invalid("font has no layer".into()))?;
        let mut width: i32 = 0;
        let mut previous = None;
        for character in text.chars() {
            let glyph = layer.glyphs.get(&character).ok_or_else(|| {
                FontError::Invalid(format!("font has no glyph for {character:?}"))
            })?;
            if let Some(before) = previous {
                width = width
                    .checked_add(
                        layer.spacing
                            + layer
                                .kerning
                                .get(&(before, character))
                                .copied()
                                .unwrap_or(0),
                    )
                    .ok_or_else(|| FontError::Invalid("text width overflow".into()))?;
            }
            width = width
                .checked_add(glyph.advance)
                .ok_or_else(|| FontError::Invalid("text width overflow".into()))?;
            previous = Some(character);
        }
        Ok(width)
    }
}

fn decode_windows_1252(bytes: &[u8]) -> Result<String, FontError> {
    // The owned font scripts contain single-byte extended glyphs (0xA1..0xFF).
    // Reject the five undefined Windows-1252 bytes instead of replacing glyphs.
    const EXTRA: [Option<char>; 32] = [
        Some('€'),
        None,
        Some('‚'),
        Some('ƒ'),
        Some('„'),
        Some('…'),
        Some('†'),
        Some('‡'),
        Some('ˆ'),
        Some('‰'),
        Some('Š'),
        Some('‹'),
        Some('Œ'),
        None,
        Some('Ž'),
        None,
        None,
        Some('‘'),
        Some('’'),
        Some('“'),
        Some('”'),
        Some('•'),
        Some('–'),
        Some('—'),
        Some('˜'),
        Some('™'),
        Some('š'),
        Some('›'),
        Some('œ'),
        None,
        Some('ž'),
        Some('Ÿ'),
    ];
    let mut decoded = String::with_capacity(bytes.len());
    for (index, &byte) in bytes.iter().enumerate() {
        let character = match byte {
            0x80..=0x9f => EXTRA[usize::from(byte - 0x80)].ok_or_else(|| {
                FontError::Invalid(format!(
                    "undefined Windows-1252 byte 0x{byte:02x} at offset {index}"
                ))
            })?,
            _ => char::from(byte),
        };
        decoded.push(character);
    }
    Ok(decoded)
}

#[derive(Clone, Debug)]
enum Expr {
    Atom(String),
    List(Vec<Expr>),
}

#[derive(Clone, Debug)]
enum Token {
    Atom(String),
    Open,
    Close,
    Comma,
    End,
}

fn safe_relative(value: &str) -> Result<PathBuf, FontError> {
    let path = PathBuf::from(value.replace('\\', "/"));
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(FontError::Invalid(format!("unsafe font path {value:?}")));
    }
    Ok(path)
}

fn lex(script: &str) -> Result<Vec<Token>, FontError> {
    let bytes = script.as_bytes();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if byte.is_ascii_whitespace() {
            at += 1;
            continue;
        }
        let punctuation = match byte {
            b'(' => Some(Token::Open),
            b')' => Some(Token::Close),
            b',' => Some(Token::Comma),
            b';' => Some(Token::End),
            _ => None,
        };
        if let Some(token) = punctuation {
            tokens.push(token);
            at += 1;
            continue;
        }
        if byte == b'\'' || byte == b'"' {
            let quote = byte;
            at += 1;
            let mut value = String::new();
            let mut closed = false;
            while at < bytes.len() {
                let current = bytes[at];
                at += 1;
                if current == quote {
                    closed = true;
                    break;
                }
                if current == b'\\' {
                    if at == bytes.len() {
                        return Err(FontError::Invalid("unfinished string escape".into()));
                    }
                    if !bytes[at].is_ascii() {
                        return Err(FontError::Invalid("non-ASCII font string".into()));
                    }
                    value.push(char::from(bytes[at]));
                    at += 1;
                } else {
                    if current.is_ascii() {
                        value.push(char::from(current));
                    } else {
                        let start = at - 1;
                        let character = script[start..]
                            .chars()
                            .next()
                            .ok_or_else(|| FontError::Invalid("invalid font string".into()))?;
                        value.push(character);
                        at = start + character.len_utf8();
                    }
                }
            }
            if !closed {
                return Err(FontError::Invalid("unclosed string".into()));
            }
            tokens.push(Token::Atom(value));
            continue;
        }
        let start = at;
        while at < bytes.len()
            && !bytes[at].is_ascii_whitespace()
            && !b"(),;\"'".contains(&bytes[at])
        {
            at += 1;
        }
        if at == start {
            return Err(FontError::Invalid(format!("bad byte at {at}")));
        }
        let word = std::str::from_utf8(&bytes[start..at])
            .map_err(|_| FontError::Invalid("non-UTF8 atom".into()))?;
        tokens.push(Token::Atom(word.to_owned()));
    }
    Ok(tokens)
}

fn statements(tokens: &[Token]) -> Result<Vec<(String, Vec<Expr>)>, FontError> {
    let mut at = 0;
    let mut result = Vec::new();
    while at < tokens.len() {
        let Token::Atom(command) = &tokens[at] else {
            return Err(FontError::Invalid("expected command".into()));
        };
        at += 1;
        let mut args = Vec::new();
        loop {
            match tokens.get(at) {
                Some(Token::End) => {
                    at += 1;
                    break;
                }
                Some(Token::Comma) => return Err(FontError::Invalid("comma outside list".into())),
                None => return Err(FontError::Invalid(format!("unterminated {command}"))),
                _ => args.push(expr(tokens, &mut at)?),
            }
        }
        result.push((command.clone(), args));
    }
    Ok(result)
}

fn expr(tokens: &[Token], at: &mut usize) -> Result<Expr, FontError> {
    match tokens.get(*at) {
        Some(Token::Atom(value)) => {
            *at += 1;
            Ok(Expr::Atom(value.clone()))
        }
        Some(Token::Open) => {
            *at += 1;
            let mut items = Vec::new();
            loop {
                match tokens.get(*at) {
                    Some(Token::Close) => {
                        *at += 1;
                        return Ok(Expr::List(items));
                    }
                    Some(Token::Comma) => {
                        *at += 1;
                    }
                    Some(Token::End) | None => {
                        return Err(FontError::Invalid("unclosed list".into()));
                    }
                    _ => items.push(expr(tokens, at)?),
                }
            }
        }
        _ => Err(FontError::Invalid("expected value or list".into())),
    }
}

fn resolve(expression: &Expr, definitions: &HashMap<String, Expr>) -> Result<Expr, FontError> {
    match expression {
        Expr::Atom(name) => definitions
            .get(name)
            .cloned()
            .ok_or_else(|| FontError::Invalid(format!("undefined list {name}"))),
        Expr::List(items) => Ok(Expr::List(items.clone())),
    }
}

fn atom(expression: &Expr) -> Result<&str, FontError> {
    match expression {
        Expr::Atom(value) => Ok(value),
        Expr::List(_) => Err(FontError::Invalid("expected scalar".into())),
    }
}

fn integer(expression: &Expr) -> Result<i32, FontError> {
    let value = atom(expression)?;
    value
        .parse()
        .map_err(|_| FontError::Invalid(format!("expected integer, got {value:?}")))
}

fn list(expression: Expr) -> Result<Vec<Expr>, FontError> {
    match expression {
        Expr::List(items) => Ok(items),
        Expr::Atom(value) => Err(FontError::Invalid(format!("expected list, got {value:?}"))),
    }
}

fn single_char(expression: &Expr) -> Result<char, FontError> {
    let mut chars = atom(expression)?.chars();
    let character = chars
        .next()
        .ok_or_else(|| FontError::Invalid("empty character".into()))?;
    if chars.next().is_some() {
        return Err(FontError::Invalid("expected one character".into()));
    }
    Ok(character)
}

fn expect_count(command: &str, args: &[Expr], count: usize) -> Result<(), FontError> {
    if args.len() != count {
        return Err(FontError::Invalid(format!(
            "{command} expects {count} arguments, got {}",
            args.len()
        )));
    }
    Ok(())
}

fn check_lengths(command: &str, left: &[Expr], right: &[Expr]) -> Result<(), FontError> {
    if left.len() != right.len() {
        return Err(FontError::Invalid(format!(
            "{command} list length mismatch: {} versus {}",
            left.len(),
            right.len()
        )));
    }
    Ok(())
}
