use super::notes;
use super::operations::source_screenshots_dir;
use crate::State;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const BUNDLE_FORMAT: &str = "starryisle-screenshots";
pub const BUNDLE_VERSION: u32 = 1;
const MANIFEST_ENTRY: &str = "manifest.json";
const NOTES_ENTRY: &str = "notes.json";
const SHOTS_PREFIX: &str = "screenshots/";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    pub format: String,
    pub version: u32,
    pub instance_name: String,
    pub exported_at: i64,
    pub screenshot_count: usize,
    pub note_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleNoteEntry {
    pub file_name: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleExportReport {
    pub screenshot_count: usize,
    pub note_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleImportReport {
    pub imported: usize,
    pub skipped: usize,
    pub overwritten: usize,
    pub renamed: usize,
    pub notes_written: usize,
    pub notes_skipped: usize,
}

#[derive(Debug, Clone, Copy)]
enum ConflictPolicy {
    Skip,
    Overwrite,
    Rename,
}

impl ConflictPolicy {
    fn parse(value: &str) -> crate::Result<Self> {
        match value {
            "skip" => Ok(Self::Skip),
            "overwrite" => Ok(Self::Overwrite),
            "rename" => Ok(Self::Rename),
            other => Err(crate::ErrorKind::InputError(format!(
                "Unknown conflict policy: {other}"
            ))
            .into()),
        }
    }
}

async fn screenshots_source(
    instance_id: &str,
    state: &State,
) -> crate::Result<(PathBuf, String)> {
    let source = crate::state::instances::adapters::sqlite::instance_rows::
        get_instance_screenshot_source(instance_id, &state.pool)
        .await?
        .ok_or_else(|| crate::ErrorKind::InputError("Unknown instance".to_string()))?;
    let dir = source_screenshots_dir(state, &source).await?;
    Ok((dir, source.name))
}

pub async fn export_instance_bundle(
    instance_id: &str,
    export_path: PathBuf,
) -> crate::Result<BundleExportReport> {
    let state = State::get().await?;
    let (dir, instance_name) = screenshots_source(instance_id, &state).await?;
    let shot_notes = notes::list_screenshot_notes(Some(instance_id)).await?;

    let mut files = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.to_lowercase().ends_with(".png") {
                continue;
            }
            files.push((name, entry.path()));
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let note_entries: Vec<BundleNoteEntry> = shot_notes
        .into_iter()
        .map(|note| BundleNoteEntry {
            file_name: note.file_name,
            note: note.note,
        })
        .collect();

    let manifest = BundleManifest {
        format: BUNDLE_FORMAT.to_string(),
        version: BUNDLE_VERSION,
        instance_name,
        exported_at: chrono::Utc::now().timestamp_millis(),
        screenshot_count: files.len(),
        note_count: note_entries.len(),
    };

    let report = BundleExportReport {
        screenshot_count: files.len(),
        note_count: note_entries.len(),
    };

    tokio::task::spawn_blocking(move || -> crate::Result<()> {
        let file = std::fs::File::create(&export_path)?;
        let mut writer = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        writer.start_file(MANIFEST_ENTRY, options).map_err(std::io::Error::from)?;
        writer.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;

        writer.start_file(NOTES_ENTRY, options).map_err(std::io::Error::from)?;
        writer.write_all(serde_json::to_string_pretty(&note_entries)?.as_bytes())?;

        for (name, path) in files {
            writer.start_file(format!("{SHOTS_PREFIX}{name}"), options).map_err(std::io::Error::from)?;
            let mut source = std::fs::File::open(&path)?;
            std::io::copy(&mut source, &mut writer)?;
        }

        writer.finish().map_err(std::io::Error::from)?;
        Ok(())
    })
    .await??;

    Ok(report)
}

fn safe_relative_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }
    if result.as_os_str().is_empty() {
        None
    } else {
        Some(result)
    }
}

pub async fn import_instance_bundle(
    instance_id: &str,
    archive_path: PathBuf,
    conflict: &str,
) -> crate::Result<BundleImportReport> {
    let policy = ConflictPolicy::parse(conflict)?;
    let state = State::get().await?;
    let (dir, _) = screenshots_source(instance_id, &state).await?;
    std::fs::create_dir_all(&dir)?;

    let (mut report, pending_notes) = tokio::task::spawn_blocking({
        let dir = dir.clone();
        move || -> crate::Result<(BundleImportReport, Vec<BundleNoteEntry>)> {
            let file = std::fs::File::open(&archive_path)?;
            let mut archive = ZipArchive::new(file).map_err(std::io::Error::from)?;

            let manifest_text = {
                let mut entry = archive.by_name(MANIFEST_ENTRY).map_err(|_| {
                    crate::ErrorKind::InputError(
                        "Archive is missing manifest.json".to_string(),
                    )
                })?;
                let mut text = String::new();
                entry.read_to_string(&mut text)?;
                text
            };
            let manifest: BundleManifest = serde_json::from_str(&manifest_text)?;
            if manifest.format != BUNDLE_FORMAT {
                return Err(crate::ErrorKind::InputError(
                    "Archive was not exported by StarryIsle".to_string(),
                )
                .into());
            }
            if manifest.version > BUNDLE_VERSION {
                return Err(crate::ErrorKind::InputError(
                    "Archive requires a newer launcher version".to_string(),
                )
                .into());
            }

            let mut notes_text = String::new();
            if let Ok(mut entry) = archive.by_name(NOTES_ENTRY) {
                entry.read_to_string(&mut notes_text)?;
            }

            let mut report = BundleImportReport {
                imported: 0,
                skipped: 0,
                overwritten: 0,
                renamed: 0,
                notes_written: 0,
                notes_skipped: 0,
            };

            for index in 0..archive.len() {
                let mut entry = archive.by_index(index).map_err(std::io::Error::from)?;
                let Some(relative) = safe_relative_path(entry.name()) else {
                    continue;
                };
                let Ok(name) = relative.strip_prefix(SHOTS_PREFIX) else {
                    continue;
                };
                if name.as_os_str().is_empty() {
                    continue;
                }

                let mut target = dir.join(name);
                if target.exists() {
                    match policy {
                        ConflictPolicy::Skip => {
                            report.skipped += 1;
                            continue;
                        }
                        ConflictPolicy::Overwrite => report.overwritten += 1,
                        ConflictPolicy::Rename => {
                            let stem = name.file_stem().unwrap_or_default().to_string_lossy();
                            let ext = name
                                .extension()
                                .map(|value| value.to_string_lossy().to_string())
                                .unwrap_or_else(|| "png".to_string());
                            let mut counter = 1;
                            loop {
                                let candidate = dir.join(format!("{stem} ({counter}).{ext}"));
                                if !candidate.exists() {
                                    target = candidate;
                                    break;
                                }
                                counter += 1;
                            }
                            report.renamed += 1;
                        }
                    }
                }

                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut out = std::fs::File::create(&target)?;
                std::io::copy(&mut entry, &mut out)?;
                report.imported += 1;
            }

            let candidates: Vec<BundleNoteEntry> = if notes_text.trim().is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&notes_text).unwrap_or_default()
            };
            let mut pending = Vec::new();
            for note in candidates {
                if dir.join(&note.file_name).exists() {
                    pending.push(note);
                } else {
                    report.notes_skipped += 1;
                }
            }

            Ok((report, pending))
        }
    })
    .await??;

    for note in pending_notes {
        notes::set_screenshot_note_with_pool(
            instance_id,
            &note.file_name,
            &note.note,
            &state.pool,
        )
        .await?;
        report.notes_written += 1;
    }

    super::reconciliation::reconcile_screenshots_with_state(instance_id, &state).await?;

    Ok(report)
}
