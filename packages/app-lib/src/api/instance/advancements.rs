// Reads saved worlds and their advancement progress from an instance.
//
// The game stores one JSON file per player under
// `saves/<world>/advancements/<uuid>.json`, holding the ids of the advancements
// that player has finished together with the criteria timestamps.

use super::paths::get_full_path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletedAdvancement {
    pub id: String,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRef {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSummary {
    pub name: String,
    pub last_played: Option<i64>,
    pub players: Vec<PlayerRef>,
    pub has_advancements: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldAdvancements {
    pub player: Option<String>,
    pub players: Vec<PlayerRef>,
    pub completed: Vec<CompletedAdvancement>,
}

/// Maps player uuids to names using the instance's `usercache.json`.
fn player_names(
    instance_root: &Path,
) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let Ok(text) =
        std::fs::read_to_string(instance_root.join("usercache.json"))
    else {
        return map;
    };
    let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&text)
    else {
        return map;
    };
    for entry in entries {
        let Some(uuid) = entry.get("uuid").and_then(|value| value.as_str())
        else {
            continue;
        };
        let Some(name) = entry.get("name").and_then(|value| value.as_str())
        else {
            continue;
        };
        map.insert(uuid.to_string(), name.to_string());
    }
    map
}

fn to_player_refs(
    ids: Vec<String>,
    names: &std::collections::HashMap<String, String>,
) -> Vec<PlayerRef> {
    ids.into_iter()
        .map(|id| PlayerRef {
            name: names.get(&id).cloned(),
            id,
        })
        .collect()
}

/// Reads the game's own `LastPlayed` stamp from a world's `level.dat`, falling
/// back to `None` when the file is missing or unreadable.
fn last_played_from_level(world_dir: &Path) -> Option<i64> {
    let file = std::fs::File::open(world_dir.join("level.dat")).ok()?;
    let reader = flate2::read::GzDecoder::new(std::io::BufReader::new(file));
    let (root, _) = quartz_nbt::io::read_nbt(
        &mut std::io::BufReader::new(reader),
        quartz_nbt::io::Flavor::Uncompressed,
    )
    .ok()?;
    let data = root.get::<_, &quartz_nbt::NbtCompound>("Data").ok()?;
    data.get::<_, i64>("LastPlayed").ok()
}
async fn saves_dir(instance_id: &str) -> crate::Result<PathBuf> {
    Ok(get_full_path(instance_id).await?.join("saves"))
}

/// Worlds keep player advancement files in `advancements/`, while newer game
/// versions moved them to `players/advancements/`; both are read here.
fn advancement_dirs(world_dir: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for candidate in ["advancements", "players/advancements"] {
        let dir = world_dir.join(candidate);
        if dir.is_dir() {
            dirs.push(dir);
        }
    }
    dirs
}

fn player_ids(world_dir: &Path) -> Vec<String> {
    let mut ids = Vec::new();
    for dir in advancement_dirs(world_dir) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json")
            {
                continue;
            }
            if let Some(stem) =
                path.file_stem().and_then(|value| value.to_str())
            {
                if !ids.iter().any(|existing| existing == stem) {
                    ids.push(stem.to_string());
                }
            }
        }
    }
    ids.sort();
    ids
}

fn advancement_file(world_dir: &Path, player: &str) -> Option<PathBuf> {
    for dir in advancement_dirs(world_dir) {
        let file = dir.join(format!("{player}.json"));
        if file.is_file() {
            return Some(file);
        }
    }
    None
}

fn timestamp_of(entry: &serde_json::Value) -> Option<i64> {
    let criteria = entry.get("criteria")?.as_object()?;
    let mut earliest: Option<i64> = None;
    for value in criteria.values() {
        let Some(text) = value.as_str() else {
            continue;
        };
        let Ok(parsed) = DateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S %z")
        else {
            continue;
        };
        let millis = parsed.with_timezone(&Utc).timestamp_millis();
        earliest = Some(match earliest {
            Some(current) => current.min(millis),
            None => millis,
        });
    }
    earliest
}

pub async fn list_worlds(
    instance_id: &str,
) -> crate::Result<Vec<WorldSummary>> {
    let root = get_full_path(instance_id).await?;
    let names = player_names(&root);
    let dir = root.join("saves");
    tokio::task::spawn_blocking(move || -> crate::Result<Vec<WorldSummary>> {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error.into()),
        };

        let mut worlds = Vec::new();
        for entry in entries.flatten() {
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            let players = to_player_refs(player_ids(&path), &names);
            let last_played = last_played_from_level(&path).or_else(|| {
                entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.modified().ok())
                    .map(|time| DateTime::<Utc>::from(time).timestamp_millis())
            });
            worlds.push(WorldSummary {
                name,
                last_played,
                has_advancements: !players.is_empty(),
                players,
            });
        }

        worlds.sort_by(|left, right| right.last_played.cmp(&left.last_played));
        Ok(worlds)
    })
    .await?
}

pub async fn world_advancements(
    instance_id: &str,
    world: &str,
    player: Option<&str>,
) -> crate::Result<WorldAdvancements> {
    let root = get_full_path(instance_id).await?;
    let names = player_names(&root);
    let world = world.to_string();
    let dir = saves_dir(instance_id).await?.join(&world);
    let requested = player.map(str::to_string);
    let ids = player_ids(&dir);
    tokio::task::spawn_blocking(move || -> crate::Result<WorldAdvancements> {
        let chosen = requested
            .filter(|value| ids.contains(value))
            .or_else(|| ids.first().cloned());
        let players = to_player_refs(ids, &names);

        let mut completed = Vec::new();
        if let Some(player) = &chosen {
            if let Some(file) = advancement_file(&dir, player) {
                if let Ok(text) = std::fs::read_to_string(&file) {
                    if let Ok(serde_json::Value::Object(map)) =
                        serde_json::from_str(&text)
                    {
                        for (id, entry) in map {
                            // Recipe unlocks share the file but are not advancements, and
                            // the file also carries bookkeeping keys such as DataVersion.
                            if !id.contains(':') {
                                continue;
                            }
                            if id.starts_with("minecraft:recipes/")
                                || id.contains(":recipes/")
                            {
                                continue;
                            }
                            let done = entry
                                .get("done")
                                .and_then(|value| value.as_bool())
                                .unwrap_or(true);
                            if !done {
                                continue;
                            }
                            completed.push(CompletedAdvancement {
                                id,
                                completed_at: timestamp_of(&entry),
                            });
                        }
                    }
                }
            }
        }
        completed.sort_by(|left, right| left.id.cmp(&right.id));

        Ok(WorldAdvancements {
            player: chosen,
            players,
            completed,
        })
    })
    .await?
}

/// Every advancement completed in a world by any player, keeping the earliest
/// completion time per advancement. Used by the statistics page, which merges
/// the whole library instead of looking at one player at a time.
pub async fn world_completed_all_players(
    instance_id: &str,
    world: &str,
) -> crate::Result<Vec<CompletedAdvancement>> {
    let dir = saves_dir(instance_id).await?.join(world);
    tokio::task::spawn_blocking(
        move || -> crate::Result<Vec<CompletedAdvancement>> {
            let mut earliest: std::collections::HashMap<String, Option<i64>> =
                std::collections::HashMap::new();

            for player in player_ids(&dir) {
                let Some(file) = advancement_file(&dir, &player) else {
                    continue;
                };
                let Ok(text) = std::fs::read_to_string(&file) else {
                    continue;
                };
                let Ok(serde_json::Value::Object(map)) =
                    serde_json::from_str(&text)
                else {
                    continue;
                };
                for (id, entry) in map {
                    // Recipe unlocks share the file but are not advancements, and the file
                    // also carries bookkeeping keys such as DataVersion.
                    if !id.contains(':')
                        || id.starts_with("minecraft:recipes/")
                        || id.contains(":recipes/")
                    {
                        continue;
                    }
                    let done = entry
                        .get("done")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(true);
                    if !done {
                        continue;
                    }
                    let stamp = timestamp_of(&entry);
                    earliest
                        .entry(id)
                        .and_modify(|current| {
                            *current = match (*current, stamp) {
                                (Some(existing), Some(candidate)) => {
                                    Some(existing.min(candidate))
                                }
                                (None, Some(candidate)) => Some(candidate),
                                (existing, None) => existing,
                            };
                        })
                        .or_insert(stamp);
                }
            }

            let mut completed: Vec<CompletedAdvancement> = earliest
                .into_iter()
                .map(|(id, completed_at)| CompletedAdvancement {
                    id,
                    completed_at,
                })
                .collect();
            completed.sort_by(|left, right| left.id.cmp(&right.id));
            Ok(completed)
        },
    )
    .await?
}

/// Absolute path of the newest version jar the launcher has downloaded, used to
/// pull advancement icons out of the player's own game files. Versions live in
/// `versions/<name>/<name>.jar`.
/// Absolute path of the game jar that belongs to an instance. The instance's own
/// configuration names the version directory, so two instances on different game
/// versions do not share a catalogue.
pub async fn version_jar(
    instance_id: Option<&str>,
) -> crate::Result<Option<PathBuf>> {
    let state = State::get().await?;
    let versions = state.directories.versions_dir();

    let mut preferred: Option<String> = None;
    if let Some(id) = instance_id {
        if let Ok(root) = get_full_path(id).await {
            if let Ok(text) =
                std::fs::read_to_string(root.join("axolotl_config.json"))
            {
                if let Ok(value) =
                    serde_json::from_str::<serde_json::Value>(&text)
                {
                    let content = value.get("content_set");
                    let game = content
                        .and_then(|set| set.get("game_version"))
                        .and_then(|item| item.as_str());
                    let loader = content
                        .and_then(|set| set.get("loader_version"))
                        .and_then(|item| item.as_str());
                    preferred = match (game, loader) {
                        (Some(game), Some(loader)) => {
                            Some(format!("{game}-{loader}"))
                        }
                        (Some(game), None) => Some(game.to_string()),
                        _ => None,
                    };
                }
            }
        }
    }

    tokio::task::spawn_blocking(move || -> crate::Result<Option<PathBuf>> {
        let Ok(entries) = std::fs::read_dir(&versions) else {
            return Ok(None);
        };
        let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
        let mut matched: Option<PathBuf> = None;
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let folder_name = dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string();
            let Ok(files) = std::fs::read_dir(&dir) else {
                continue;
            };
            for file in files.flatten() {
                let path = file.path();
                if path.extension().and_then(|value| value.to_str())
                    != Some("jar")
                {
                    continue;
                }
                let matches_instance = preferred
                    .as_deref()
                    .map(|wanted| {
                        folder_name == wanted || folder_name.starts_with(wanted)
                    })
                    .unwrap_or(false);
                if matches_instance {
                    matched = Some(path.clone());
                }
                let Ok(meta) = file.metadata() else {
                    continue;
                };
                let Ok(modified) = meta.modified() else {
                    continue;
                };
                if newest
                    .as_ref()
                    .map(|(time, _)| modified > *time)
                    .unwrap_or(true)
                {
                    newest = Some((modified, path));
                }
            }
        }
        Ok(matched.or_else(|| newest.map(|(_, path)| path)))
    })
    .await?
}
