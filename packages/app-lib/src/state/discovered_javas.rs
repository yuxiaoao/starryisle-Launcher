use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::state::store::Atom;
use crate::state::store::Collection as _;
use crate::state::store::Handle;
use crate::state::store::discovered_javas::{
    DiscoveredJavaRecord, DiscoveredJavas,
};

use super::JavaVersion;

/// A Java installation found by a system scan, cached together with the
/// file signature of its executable so staleness can be detected cheaply.
#[derive(Debug, Clone)]
pub struct DiscoveredJava {
    pub java: JavaVersion,
    pub file_size: i64,
    pub file_mtime_ms: i64,
}

/// Returns (size, mtime in milliseconds) of the file at `path`, or None
/// if it does not exist or cannot be read.
pub fn java_file_signature(path: &Path) -> Option<(i64, i64)> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    let mtime_ms = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis() as i64;
    Some((metadata.len() as i64, mtime_ms))
}

impl DiscoveredJava {
    /// Builds a cache entry for a verified Java installation, stamping the
    /// current signature of its executable. Returns None if the executable
    /// can no longer be read.
    pub fn from_java(java: JavaVersion) -> Option<Self> {
        let (file_size, file_mtime_ms) =
            java_file_signature(Path::new(&java.path))?;
        Some(Self {
            java,
            file_size,
            file_mtime_ms,
        })
    }

    fn from_record(path: String, record: DiscoveredJavaRecord) -> Self {
        Self {
            java: JavaVersion {
                parsed_version: record.major_version,
                version: record.full_version,
                architecture: record.architecture,
                path,
                distribution: record.distribution,
            },
            file_size: record.file_size,
            file_mtime_ms: record.file_mtime_ms,
        }
    }

    fn to_record(&self) -> DiscoveredJavaRecord {
        DiscoveredJavaRecord {
            major_version: self.java.parsed_version,
            full_version: self.java.version.clone(),
            architecture: self.java.architecture.clone(),
            distribution: self.java.distribution.clone(),
            file_size: self.file_size,
            file_mtime_ms: self.file_mtime_ms,
        }
    }

    pub async fn get_all(handle: &Handle) -> crate::Result<Vec<Self>> {
        let mut entries = DiscoveredJavas::get_all(handle)
            .await?
            .into_iter()
            .map(|(path, record)| Self::from_record(path, record))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.java
                .parsed_version
                .cmp(&right.java.parsed_version)
                .then_with(|| left.java.path.cmp(&right.java.path))
        });

        Ok(entries)
    }

    pub async fn upsert(&self, handle: &Handle) -> crate::Result<()> {
        DiscoveredJavas::set(handle, &self.java.path, Some(&self.to_record()))
            .await
    }

    pub(crate) async fn upsert_in(&self, unit: &mut Atom) -> crate::Result<()> {
        DiscoveredJavas::atom_set(
            unit,
            &self.java.path,
            Some(&self.to_record()),
        )
        .await
    }

    pub async fn remove(path: &str, handle: &Handle) -> crate::Result<()> {
        DiscoveredJavas::set(handle, &path.to_string(), None).await
    }

    /// Replaces the entire cache with the results of a fresh scan.
    pub async fn replace_all(
        handle: &Handle,
        entries: &[Self],
    ) -> crate::Result<()> {
        let records = entries
            .iter()
            .map(|entry| (entry.java.path.clone(), entry.to_record()))
            .collect::<Vec<_>>();

        DiscoveredJavas::set_all(handle, &records).await
    }
}
