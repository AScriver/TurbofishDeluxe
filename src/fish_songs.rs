//! Read-only singing-fish scores and bounded note scheduling.
//!
//! The installed `fishsongs/*.txt` files supply scores. This module emits
//! effect requests; the application's audio owner chooses how to render them.
//! Format and timing are adapted from pinned W1 `FishSongMgr.cpp` (secondary
//! source); audible sample pitch, overlap, and retail timing remain unverified.

use std::{collections::BTreeMap, fmt, fs, io::Read, path::Path};

const MAX_FILE_BYTES: u64 = 64 * 1024;
const MAX_SONG_FILES: usize = 128;
const MAX_NOTES_PER_VOICE: usize = 8_192;
const MAX_SCORE_MS: u64 = 3_600_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SongError {
    pub line: usize,
    pub message: String,
}

impl SongError {
    fn at(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for SongError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(formatter, "{}", self.message)
        } else {
            write!(formatter, "line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for SongError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoreNote {
    /// Start within this voice, in milliseconds. The source parser scales the
    /// written duration at parse time; the playback manager starts at 0.
    pub at_ms: u32,
    pub duration_ms: u32,
    pub pitch: Option<f32>,
    pub gain: f32,
}

#[derive(Clone, Debug)]
pub struct SongData {
    pub file_name: String,
    pub voices: [Vec<ScoreNote>; 3],
    pub properties: BTreeMap<String, String>,
}

impl SongData {
    pub fn property(&self, key: &str) -> Option<&str> {
        self.properties
            .get(&key.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn is_long(&self) -> bool {
        self.property("long").is_some()
    }

    pub fn from_file(path: &Path) -> Result<Self, SongError> {
        let metadata = fs::metadata(path).map_err(|error| SongError::at(0, error.to_string()))?;
        if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
            return Err(SongError::at(
                0,
                "song must be a regular file of at most 64 KiB",
            ));
        }
        let file = fs::File::open(path).map_err(|error| SongError::at(0, error.to_string()))?;
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        file.take(MAX_FILE_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| SongError::at(0, error.to_string()))?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(SongError::at(0, "song exceeds 64 KiB"));
        }
        let source =
            String::from_utf8(bytes).map_err(|error| SongError::at(0, error.to_string()))?;
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| SongError::at(0, "invalid song file name"))?;
        Self::parse(name, &source)
    }

    pub fn parse(file_name: &str, source: &str) -> Result<Self, SongError> {
        if source.len() as u64 > MAX_FILE_BYTES {
            return Err(SongError::at(0, "song exceeds 64 KiB"));
        }
        let mut song = Self {
            file_name: file_name.to_owned(),
            voices: std::array::from_fn(|_| Vec::new()),
            properties: BTreeMap::new(),
        };
        let mut voice = 0usize;
        let mut speed = 3.0_f64;
        let mut global_shift = 0.0_f32;
        let mut local_shifts = [0.0_f32; 3];
        let mut gains = [1.0_f32; 3];
        let mut elapsed = [0_u64; 3];
        let mut skip = false;
        for (index, raw) in source.lines().enumerate() {
            let line_number = index + 1;
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(command) = line.strip_prefix('*') {
                let mut words = command.split_whitespace();
                let key = words.next().unwrap_or_default().to_ascii_lowercase();
                let value = words.next();
                if key == "on" || key == "off" {
                    skip = key == "off";
                    continue;
                }
                if key == "skip" {
                    skip = value
                        .ok_or_else(|| SongError::at(line_number, "missing skip value"))?
                        .eq_ignore_ascii_case("true");
                    continue;
                }
                if skip {
                    continue;
                }
                let value =
                    value.ok_or_else(|| SongError::at(line_number, "missing command value"))?;
                match key.as_str() {
                    "line" => voice = parse_voice(value, line_number)?,
                    "rest" => {
                        let other = parse_voice(value, line_number)?;
                        let delta = elapsed[other].saturating_sub(elapsed[voice]);
                        if delta > 0 {
                            push_note(
                                &mut song.voices[voice],
                                &mut elapsed[voice],
                                None,
                                1.0,
                                delta,
                                line_number,
                            )?;
                        }
                    }
                    "attrib" => {
                        let property = value.to_ascii_lowercase();
                        let text = line
                            .split_once('=')
                            .map(|(_, text)| text.trim())
                            .unwrap_or("");
                        song.properties.insert(property, text.to_owned());
                    }
                    "speed" => {
                        speed = parse_finite(value, line_number, "speed")?;
                        if speed <= 0.0 || speed > 10_000.0 {
                            return Err(SongError::at(line_number, "speed out of range"));
                        }
                    }
                    "volume" => {
                        gains[voice] = parse_finite(value, line_number, "volume")? as f32;
                        if !(0.0..=1.0).contains(&gains[voice]) {
                            return Err(SongError::at(line_number, "volume out of range"));
                        }
                    }
                    "shift" => global_shift = parse_shift(value, line_number)?,
                    "localshift" => local_shifts[voice] = parse_shift(value, line_number)?,
                    _ => {
                        return Err(SongError::at(
                            line_number,
                            format!("unknown command: {key}"),
                        ));
                    }
                }
                continue;
            }
            if skip {
                continue;
            }
            for token in line
                .split(',')
                .map(str::trim)
                .filter(|token| !token.is_empty())
            {
                let (pitch, units) = parse_note(token, line_number)?;
                let scaled = (units as f64 * speed).round();
                if !scaled.is_finite() || scaled > MAX_SCORE_MS as f64 {
                    return Err(SongError::at(line_number, "note duration out of range"));
                }
                let shifted = pitch.map(|pitch| pitch + global_shift + local_shifts[voice]);
                push_note(
                    &mut song.voices[voice],
                    &mut elapsed[voice],
                    shifted,
                    gains[voice],
                    scaled as u64,
                    line_number,
                )?;
            }
        }
        // W1 identifies short/long pairs by file suffix, then adds this
        // property; score attributes alone do not classify the pair.
        if let Some(base) = file_name.to_ascii_lowercase().strip_suffix("_long.txt") {
            song.properties.insert("long".into(), base.into());
        } else if let Some(base) = file_name.to_ascii_lowercase().strip_suffix("_short.txt") {
            song.properties.insert("short".into(), base.into());
        }
        Ok(song)
    }
}

fn parse_voice(value: &str, line: usize) -> Result<usize, SongError> {
    match value {
        "1" => Ok(0),
        "2" => Ok(1),
        "3" => Ok(2),
        _ => Err(SongError::at(line, "voice must be 1, 2, or 3")),
    }
}

fn parse_finite(value: &str, line: usize, name: &str) -> Result<f64, SongError> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| SongError::at(line, format!("invalid {name}")))?;
    if !parsed.is_finite() {
        return Err(SongError::at(line, format!("invalid {name}")));
    }
    Ok(parsed)
}

fn parse_shift(value: &str, line: usize) -> Result<f32, SongError> {
    // W1 uses atol: retain the numeric prefix in installed "0;" values,
    // while keeping checked conversion and the project's pitch bounds.
    let unsigned = value.strip_prefix(['-', '+']).unwrap_or(value);
    let digit_count = unsigned.bytes().take_while(u8::is_ascii_digit).count();
    let prefix_len = value.len() - unsigned.len() + digit_count;
    let parsed = value[..prefix_len]
        .parse::<i32>()
        .map_err(|_| SongError::at(line, "invalid pitch shift"))?;
    if !(-96..=96).contains(&parsed) {
        return Err(SongError::at(line, "pitch shift out of range"));
    }
    Ok(parsed as f32)
}

fn push_note(
    voice: &mut Vec<ScoreNote>,
    elapsed: &mut u64,
    pitch: Option<f32>,
    gain: f32,
    duration: u64,
    line: usize,
) -> Result<(), SongError> {
    if voice.len() >= MAX_NOTES_PER_VOICE || *elapsed + duration > MAX_SCORE_MS {
        return Err(SongError::at(line, "song exceeds note or duration limit"));
    }
    voice.push(ScoreNote {
        at_ms: *elapsed as u32,
        duration_ms: duration as u32,
        pitch,
        gain,
    });
    *elapsed += duration;
    Ok(())
}

fn parse_note(token: &str, line: usize) -> Result<(Option<f32>, u32), SongError> {
    let mut parts = token.split_whitespace();
    let name = parts
        .next()
        .ok_or_else(|| SongError::at(line, "missing note"))?;
    let duration = parts
        .next()
        .ok_or_else(|| SongError::at(line, "missing duration"))?;
    if parts.next().is_some() {
        return Err(SongError::at(line, "note has extra fields"));
    }
    let mut chars = name.chars();
    let base = match chars.next().unwrap_or_default().to_ascii_lowercase() {
        'c' => Some(0),
        'd' => Some(2),
        'e' => Some(4),
        'f' => Some(5),
        'g' => Some(7),
        'a' => Some(9),
        'b' => Some(11),
        'r' => None,
        _ => return Err(SongError::at(line, "invalid note name")),
    };
    let suffix: String = chars.collect();
    let (accidental, octave) = match suffix.as_bytes() {
        // W1 returns its initialized note (pitch/duration 0) for an
        // unrecognized modifier. The installed Pathetique scores use "an3".
        [modifier, ..] if !modifier.is_ascii_digit() && !matches!(modifier, b'#' | b'b') => {
            return Ok((Some(0.0), 0));
        }
        [] => (0, 4),
        [digit] if digit.is_ascii_digit() => (0, (*digit - b'0') as i32),
        [b'#', digit] if digit.is_ascii_digit() => (1, (*digit - b'0') as i32),
        [b'b', digit] if digit.is_ascii_digit() => (-1, (*digit - b'0') as i32),
        // W1 retains octave 4 when the character after an accidental is
        // not a digit. Installed hammerklavier1_long.txt:73 uses "bbe".
        [b'#'] | [b'#', _] => (1, 4),
        [b'b'] | [b'b', _] => (-1, 4),
        _ => return Err(SongError::at(line, "invalid note octave or accidental")),
    };
    let pitch = base.map(|base| (base + accidental + (octave - 4) * 12) as f32);
    // W1's numeric branch uses atol, including installed "0k" durations.
    // Overflow still fails instead of adopting the source's unchecked cast.
    let numeric_len = duration.bytes().take_while(u8::is_ascii_digit).count();
    if numeric_len > 0 {
        let number = duration[..numeric_len]
            .parse::<u32>()
            .map_err(|_| SongError::at(line, "duration out of range"))?;
        return Ok((pitch, number));
    }
    // W1 NoteData::Parse consumes a final '+' before reaching the end.
    // Accept exactly one, as used by installed entertainer_long.txt:69;
    // leading or repeated separators still leave an invalid empty term.
    let duration = duration.strip_suffix('+').unwrap_or(duration);
    let mut total = 0_u32;
    for term in duration.split('+') {
        if term.is_empty() {
            return Err(SongError::at(line, "empty duration term"));
        }
        let mut letters = term.chars().peekable();
        let base = match letters.next().unwrap_or_default().to_ascii_lowercase() {
            'w' => 192,
            'h' => 96,
            'q' => 48,
            'e' => 24,
            's' => 12,
            't' => 6,
            'z' => 3,
            _ => return Err(SongError::at(line, "invalid duration")),
        };
        let mut step = base;
        if letters
            .peek()
            .is_some_and(|letter| letter.eq_ignore_ascii_case(&'t'))
        {
            letters.next();
            step = step * 2 / 3;
        }
        let mut amount = step;
        while letters.peek().is_some() {
            let letter = letters.next().unwrap_or_default();
            if !letter.eq_ignore_ascii_case(&'d') {
                return Err(SongError::at(line, "invalid dotted duration"));
            }
            step /= 2;
            amount += step;
        }
        total = total
            .checked_add(amount)
            .ok_or_else(|| SongError::at(line, "duration overflow"))?;
    }
    Ok((pitch, total))
}

#[derive(Default)]
pub struct SongCatalog {
    songs: Vec<SongData>,
}

impl SongCatalog {
    pub fn load_from_install(root: &Path) -> Result<Self, SongError> {
        let directory = root.join("fishsongs");
        let resolved_directory = directory
            .canonicalize()
            .map_err(|error| SongError::at(0, error.to_string()))?;
        let mut paths = Vec::new();
        for entry in
            fs::read_dir(&directory).map_err(|error| SongError::at(0, error.to_string()))?
        {
            let entry = entry.map_err(|error| SongError::at(0, error.to_string()))?;
            let path = entry.path();
            if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("txt"))
            {
                let resolved = path
                    .canonicalize()
                    .map_err(|error| SongError::at(0, error.to_string()))?;
                if !resolved.starts_with(&resolved_directory) {
                    return Err(SongError::at(
                        0,
                        format!("song path escapes installation: {}", path.display()),
                    ));
                }
                if paths.len() >= MAX_SONG_FILES {
                    return Err(SongError::at(0, "too many song files"));
                }
                paths.push(path);
            }
        }
        paths.sort();
        let mut songs = Vec::with_capacity(paths.len());
        for path in paths {
            let song = SongData::from_file(&path).map_err(|error| {
                SongError::at(error.line, format!("{}: {}", path.display(), error.message))
            })?;
            songs.push(song);
        }
        Ok(Self { songs })
    }

    pub fn songs(&self) -> &[SongData] {
        &self.songs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToneSample {
    Normal,
    High,
    SuperHigh,
    Low,
    Applause,
}

impl ToneSample {
    pub fn sound_id(self) -> &'static str {
        match self {
            Self::Normal => "SOUND_TONE",
            Self::High => "SOUND_TONEHI",
            Self::SuperHigh => "SOUND_TONESUPERHI",
            Self::Low => "SOUND_TONELO",
            Self::Applause => "SOUND_APPLAUSE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToneCommand {
    pub song_id: i32,
    pub sample: ToneSample,
    /// Semitone shift to apply to the selected installed effect sample.
    pub pitch_semitones: f32,
    pub gain: f32,
    /// Time until the next note in this voice; source sample playback itself
    /// is not cut to this duration.
    pub duration_ms: u32,
}

struct Voice {
    notes: Vec<ScoreNote>,
    next: usize,
    start_ms: u64,
}

#[derive(Default)]
pub struct FishSongPlayer {
    active_id: Option<i32>,
    voices: Vec<Voice>,
    applause_requested: bool,
    applause_at: Option<u64>,
    applause_done_at: Option<u64>,
    paused_at: Option<u64>,
}

impl FishSongPlayer {
    pub fn busy(&self) -> bool {
        !self.voices.is_empty() || self.applause_done_at.is_some()
    }
    pub fn is_song_playing(&self, song_id: i32) -> bool {
        self.active_id == Some(song_id)
    }

    pub fn start(
        &mut self,
        song: &SongData,
        song_id: i32,
        now_ms: u64,
        with_applause: bool,
    ) -> Result<(), SongError> {
        if song_id < 0 {
            return Err(SongError::at(0, "song ID must be nonnegative"));
        }
        if self.busy() {
            return Err(SongError::at(0, "another song is still playing"));
        }
        self.voices = song
            .voices
            .iter()
            .filter(|notes| !notes.is_empty())
            .map(|notes| Voice {
                notes: notes.clone(),
                next: 0,
                start_ms: now_ms,
            })
            .collect();
        if self.voices.is_empty() {
            return Err(SongError::at(0, "song has no notes"));
        }
        self.active_id = Some(song_id);
        self.applause_requested = with_applause;
        self.applause_at = None;
        self.applause_done_at = None;
        self.paused_at = None;
        Ok(())
    }

    pub fn advance(&mut self, now_ms: u64) -> Vec<ToneCommand> {
        if self.paused_at.is_some() {
            return Vec::new();
        }
        let mut emitted = Vec::new();
        let song_id = self.active_id.unwrap_or(-1);
        for voice in &mut self.voices {
            let elapsed = now_ms.saturating_sub(voice.start_ms);
            while let Some(note) = voice.notes.get(voice.next)
                && u64::from(note.at_ms) <= elapsed
            {
                if let Some(pitch) = note.pitch {
                    let (sample, offset) = if pitch > 24.0 {
                        (ToneSample::SuperHigh, -24.0)
                    } else if pitch >= 12.0 {
                        (ToneSample::High, -12.0)
                    } else if pitch <= -12.0 {
                        (ToneSample::Low, 12.0)
                    } else {
                        (ToneSample::Normal, 0.0)
                    };
                    emitted.push(ToneCommand {
                        song_id,
                        sample,
                        pitch_semitones: pitch + offset,
                        gain: note.gain,
                        duration_ms: note.duration_ms,
                    });
                }
                voice.next += 1;
            }
        }
        self.voices.retain(|voice| voice.next < voice.notes.len());
        if self.voices.is_empty() && self.active_id.take().is_some() {
            if self.applause_requested {
                self.applause_at = now_ms.checked_add(400);
                self.applause_done_at = now_ms.checked_add(500);
            }
            self.applause_requested = false;
        }
        if self.applause_at.is_some_and(|due| due <= now_ms) {
            self.applause_at = None;
            emitted.push(ToneCommand {
                song_id: -1,
                sample: ToneSample::Applause,
                pitch_semitones: 0.0,
                gain: 1.0,
                duration_ms: 100,
            });
        }
        if self.applause_done_at.is_some_and(|due| due <= now_ms) {
            self.applause_done_at = None;
        }
        emitted
    }

    pub fn pause(&mut self, now_ms: u64) {
        if self.busy() && self.paused_at.is_none() {
            self.paused_at = Some(now_ms);
        }
    }

    pub fn resume(&mut self, now_ms: u64) {
        if let Some(paused_at) = self.paused_at.take() {
            let delay = now_ms.saturating_sub(paused_at);
            for voice in &mut self.voices {
                voice.start_ms = voice.start_ms.saturating_add(delay);
            }
            if let Some(due) = &mut self.applause_at {
                *due = due.saturating_add(delay);
            }
            if let Some(due) = &mut self.applause_done_at {
                *due = due.saturating_add(delay);
            }
        }
    }

    pub fn stop(&mut self, song_id: i32) {
        if self.active_id == Some(song_id) {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.active_id = None;
        self.voices.clear();
        self.applause_requested = false;
        self.applause_at = None;
        self.applause_done_at = None;
        self.paused_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_lines_rest_and_timed_applause() {
        let score = SongData::parse(
            "tune_long.txt",
            "*speed 2\n*line 1\nc4 q, r e, c5 s\n*line 2\n*localshift 12\ng3 q\n*rest 1\na3 s\n",
        )
        .unwrap();
        assert_eq!(score.voices[0][1].at_ms, 96);
        assert_eq!(score.voices[1][1].at_ms, 96);
        assert_eq!(score.voices[1][2].at_ms, 168);
        // W1 NoteData::Parse makes A3 = 9 + (3 - 4) * 12 = -3;
        // FishSongData::Parse then adds this voice's local shift of +12.
        assert_eq!(score.voices[1][2].pitch, Some(9.0));
        assert!(score.is_long());
        let mut player = FishSongPlayer::default();
        player.start(&score, 7, 1_000, true).unwrap();
        assert_eq!(player.advance(1_000).len(), 2);
        player.pause(1_020);
        assert!(player.advance(5_000).is_empty());
        player.resume(5_000);
        assert!(player.advance(5_076).is_empty()); // both rests have begun.
        assert!(player.advance(5_096).is_empty());
        assert_eq!(player.advance(5_168).len(), 2);
        assert!(!player.is_song_playing(7));
        assert!(player.busy());
        assert!(player.advance(5_567).is_empty());
        assert_eq!(player.advance(5_568)[0].sample, ToneSample::Applause);
        assert!(player.advance(5_668).is_empty());
        assert!(!player.busy());
    }

    #[test]
    fn installed_numeric_suffixes_and_unknown_modifier_follow_source_defaults() {
        // W1 ParseCommand uses atol for pitch shifts; NoteData::Parse uses
        // atol for numeric durations and leaves its initialized fields on
        // the early return for an unknown modifier such as installed "an3".
        let score = SongData::parse(
            "numeric.txt",
            "*shift 12;\n*localshift -12;\nf4 0k, an3 q, c5 e\n",
        )
        .unwrap();
        assert_eq!(score.voices[0].len(), 3);
        assert_eq!(score.voices[0][0].pitch, Some(5.0));
        assert_eq!(score.voices[0][0].duration_ms, 0);
        assert_eq!(score.voices[0][1].pitch, Some(0.0));
        assert_eq!(score.voices[0][1].duration_ms, 0);
        assert_eq!(score.voices[0][2].pitch, Some(12.0));
        assert_eq!(score.voices[0][2].at_ms, 0);
        assert_eq!(score.voices[0][2].duration_ms, 72);
        assert!(SongData::parse("bad.txt", "c4 4294967296k").is_err());
        assert!(SongData::parse("bad.txt", "*shift 97;").is_err());
        assert!(SongData::parse("bad.txt", "*shift nope").is_err());
    }

    #[test]
    fn accidental_with_nondigit_octave_uses_default_octave() {
        // W1 NoteData::Parse initializes octave 4 and replaces it only
        // with a digit after an accidental; the installed "bbe" is B-flat4.
        let score = SongData::parse("octave.txt", "bbe 0, c#x e, bb3 e\n").unwrap();
        assert_eq!(score.voices[0][0].pitch, Some(10.0));
        assert_eq!(score.voices[0][0].duration_ms, 0);
        assert_eq!(score.voices[0][1].pitch, Some(1.0));
        assert_eq!(score.voices[0][1].at_ms, 0);
        assert_eq!(score.voices[0][1].duration_ms, 72);
        assert_eq!(score.voices[0][2].pitch, Some(-2.0));
        assert_eq!(score.voices[0][2].at_ms, 72);
    }

    #[test]
    fn trailing_duration_separator_preserves_timing() {
        // Installed entertainer_long.txt:69 uses s+s+. Pinned W1
        // NoteData::Parse consumes one trailing '+' without another term.
        let score = SongData::parse("trailing.txt", "*speed 3\ng5 s+s+, c4 q+\n").unwrap();
        assert_eq!(score.voices[0].len(), 2);
        assert_eq!(score.voices[0][0].pitch, Some(19.0));
        assert_eq!(score.voices[0][0].at_ms, 0);
        assert_eq!(score.voices[0][0].duration_ms, 72);
        assert_eq!(score.voices[0][1].at_ms, 72);
        assert_eq!(score.voices[0][1].duration_ms, 144);
    }

    #[test]
    fn malformed_score_reports_line_without_partial_playback() {
        let error = SongData::parse("bad.txt", "*line 4\nc4 q").unwrap_err();
        assert_eq!(error.line, 1);
        assert!(SongData::parse("bad.txt", "*speed 2\nc4 q, c4 nope").is_err());
        for duration in ["+", "+s", "s++", "s++s", "s+nope"] {
            let error =
                SongData::parse("bad.txt", &format!("*speed 3\nc4 {duration}\n")).unwrap_err();
            assert_eq!(error.line, 2, "{duration}");
        }
    }
}
