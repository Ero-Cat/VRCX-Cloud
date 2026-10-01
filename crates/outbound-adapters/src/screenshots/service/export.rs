use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::Path;

use vrcx_0_core::screenshots::ScreenshotZipEntry;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

use super::{Error, Result};

const EXPORT_COPY_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct ScreenshotExportOutcome {
    pub written_files: u32,
    pub skipped_files: u32,
    pub written_bytes: u64,
    pub cancelled: bool,
}

fn map_io_error(context: &str, path: &Path, error: std::io::Error) -> Error {
    Error::Custom(format!("{context} '{}' failed: {error}", path.display()))
}

pub fn total_screenshot_export_bytes(entries: &[ScreenshotZipEntry]) -> u64 {
    entries
        .iter()
        .filter_map(|entry| std::fs::metadata(&entry.source_path).ok())
        .map(|metadata| metadata.len())
        .sum()
}

pub fn write_screenshots_zip(
    entries: &[ScreenshotZipEntry],
    output_path: &Path,
    on_progress: &dyn Fn(u64, u32),
    on_finalize: &dyn Fn(),
    is_cancelled: Option<&dyn Fn() -> bool>,
) -> Result<ScreenshotExportOutcome> {
    let cancelled = || is_cancelled.is_some_and(|check| check());

    let file = File::create(output_path)
        .map_err(|error| map_io_error("Creating the archive", output_path, error))?;
    let mut archive = ZipWriter::new(BufWriter::new(file));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);

    let mut outcome = ScreenshotExportOutcome::default();
    let mut buffer = vec![0_u8; EXPORT_COPY_CHUNK_BYTES];

    for entry in entries {
        if cancelled() {
            outcome.cancelled = true;
            break;
        }

        let Ok(mut source) = File::open(&entry.source_path) else {
            outcome.skipped_files += 1;
            continue;
        };

        if let Err(error) = archive.start_file(&entry.entry_name, options) {
            drop(archive);
            let _ = std::fs::remove_file(output_path);
            return Err(Error::Custom(format!(
                "Adding '{}' to the archive failed: {error}",
                entry.entry_name
            )));
        }

        loop {
            if cancelled() {
                outcome.cancelled = true;
                break;
            }
            let read = match source.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(_) => {
                    outcome.skipped_files += 1;
                    break;
                }
            };
            if let Err(error) = archive.write_all(&buffer[..read]) {
                drop(archive);
                let _ = std::fs::remove_file(output_path);
                return Err(map_io_error("Writing the archive", output_path, error));
            }
            outcome.written_bytes += read as u64;
            on_progress(outcome.written_bytes, outcome.written_files);
        }

        if outcome.cancelled {
            break;
        }
        outcome.written_files += 1;
        on_progress(outcome.written_bytes, outcome.written_files);
    }

    if outcome.cancelled {
        drop(archive);
        let _ = std::fs::remove_file(output_path);
        return Ok(outcome);
    }

    on_finalize();
    let mut writer = archive
        .finish()
        .map_err(|error| Error::Custom(format!("Finishing the archive failed: {error}")))?;
    if let Err(error) = writer.flush() {
        let _ = std::fs::remove_file(output_path);
        return Err(map_io_error("Writing the archive", output_path, error));
    }
    let file = writer
        .into_inner()
        .map_err(|error| Error::Custom(format!("Finishing the archive failed: {error}")))?;
    if let Err(error) = file.sync_all() {
        let _ = std::fs::remove_file(output_path);
        return Err(map_io_error("Writing the archive", output_path, error));
    }

    Ok(outcome)
}
