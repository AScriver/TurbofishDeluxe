//! Shared pending-file JSON publication for project saves and observations.

use std::error::Error;

/// Serialize completely before replacing the current generation. Callers own
/// directory creation, value validation and serialization of competing writers.
pub fn write_json(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), Box<dyn Error>> {
    let pending = path.with_extension("pending");
    let bytes = serde_json::to_vec_pretty(value)?;
    std::fs::write(&pending, bytes)?;
    publish_checkpoint(&pending, path).map_err(|source| PublicationError {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(())
}

#[derive(Debug)]
struct PublicationError {
    path: std::path::PathBuf,
    source: std::io::Error,
}

impl std::fmt::Display for PublicationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Cannot publish checkpoint {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl Error for PublicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// Inspect direct or contextual I/O failures without losing their OS identity.
/// Callers retain responsibility for deciding which failures may be deferred.
pub fn io_error<'a>(mut error: &'a (dyn Error + 'static)) -> Option<&'a std::io::Error> {
    loop {
        if let Some(failure) = error.downcast_ref::<std::io::Error>() {
            return Some(failure);
        }
        error = error.source()?;
    }
}

fn publish_checkpoint(pending: &std::path::Path, path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(100);
        loop {
            match std::fs::rename(pending, path) {
                Ok(()) => return Ok(()),
                Err(error)
                    if matches!(error.raw_os_error(), Some(5 | 32 | 33))
                        && pending.is_file()
                        && path.is_file()
                        && std::time::Instant::now() < deadline =>
                {
                    // Windows may deny replacement even for a delete-shared
                    // reader. Wait briefly for observers to close; keep the old
                    // complete generation intact if the denial persists.
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                Err(error) => return Err(error),
            }
        }
    }
    #[cfg(not(windows))]
    std::fs::rename(pending, path)
}
