use super::operations::ScreenshotKey;
use crate::State;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Notes are intentionally short so a single timeline row keeps a fixed height.
pub const MAX_NOTE_CHARS: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotNote {
    pub instance_id: String,
    pub file_name: String,
    pub note: String,
    pub updated_at: i64,
}

fn validate_note(note: &str) -> crate::Result<&str> {
    let trimmed = note.trim();
    if trimmed.chars().count() > MAX_NOTE_CHARS {
        return Err(crate::ErrorKind::InputError(format!(
            "Screenshot notes are limited to {MAX_NOTE_CHARS} characters"
        ))
        .into());
    }
    Ok(trimmed)
}

pub async fn set_screenshot_note(
    instance_id: &str,
    file_name: &str,
    note: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    set_screenshot_note_with_pool(instance_id, file_name, note, &state.pool).await
}

pub(crate) async fn set_screenshot_note_with_pool(
    instance_id: &str,
    file_name: &str,
    note: &str,
    pool: &SqlitePool,
) -> crate::Result<()> {
    let trimmed = validate_note(note)?;
    if trimmed.is_empty() {
        sqlx::query(
            "DELETE FROM screenshot_notes WHERE instance_id = ? AND file_name = ?",
        )
        .bind(instance_id)
        .bind(file_name)
        .execute(pool)
        .await?;
        return Ok(());
    }

    let updated_at = chrono::Utc::now().timestamp_millis();
    sqlx::query(
        "INSERT INTO screenshot_notes (instance_id, file_name, note, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(instance_id, file_name)
         DO UPDATE SET note = excluded.note, updated_at = excluded.updated_at",
    )
    .bind(instance_id)
    .bind(file_name)
    .bind(trimmed)
    .bind(updated_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_screenshot_notes(
    instance_id: Option<&str>,
) -> crate::Result<Vec<ScreenshotNote>> {
    let state = State::get().await?;
    let rows = match instance_id {
        Some(instance_id) => {
            sqlx::query(
                "SELECT instance_id, file_name, note, updated_at
                 FROM screenshot_notes WHERE instance_id = ?",
            )
            .bind(instance_id)
            .fetch_all(&state.pool)
            .await?
        }
        None => {
            sqlx::query(
                "SELECT instance_id, file_name, note, updated_at FROM screenshot_notes",
            )
            .fetch_all(&state.pool)
            .await?
        }
    };

    Ok(rows
        .into_iter()
        .map(|row| ScreenshotNote {
            instance_id: row.get("instance_id"),
            file_name: row.get("file_name"),
            note: row.get("note"),
            updated_at: row.get("updated_at"),
        })
        .collect())
}

pub(crate) async fn delete_notes_for_keys(
    keys: &[ScreenshotKey],
    pool: &SqlitePool,
) -> crate::Result<()> {
    for key in keys {
        sqlx::query(
            "DELETE FROM screenshot_notes WHERE instance_id = ? AND file_name = ?",
        )
        .bind(&key.instance_id)
        .bind(&key.file_name)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Removes notes whose screenshot no longer exists in the index.
pub(crate) async fn prune_orphan_notes(pool: &SqlitePool) -> crate::Result<u64> {
    let result = sqlx::query(
        "DELETE FROM screenshot_notes
         WHERE NOT EXISTS (
             SELECT 1 FROM screenshots
             WHERE screenshots.instance_id = screenshot_notes.instance_id
               AND screenshots.file_name = screenshot_notes.file_name
         )",
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
