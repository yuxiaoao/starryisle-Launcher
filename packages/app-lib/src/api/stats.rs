//! Aggregated numbers for the statistics page: playtime, library size and the
//! first unlock of every achievement across all instances.

use chrono::{Datelike, Local, NaiveDate};
use serde::Serialize;
use std::collections::HashMap;

use super::instance::get_full_path;
use super::instance::{
    WorldSummary, advancement_catalog, list_worlds, world_completed_all_players,
};
use crate::state::instances::adapters::sqlite::playtime;
use crate::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayPoint {
    pub date: String,
    pub seconds: u64,
    pub top_instance: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceTotal {
    pub name: String,
    pub seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub id: String,
    pub title: String,
    pub icon_path: Option<String>,
    pub instance_name: String,
    pub world_name: String,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsOverview {
    pub total_seconds: u64,
    pub instance_count: u64,
    pub screenshot_count: u64,
    pub achievement_total: u64,
    pub days: Vec<DayPoint>,
    pub instances: Vec<InstanceTotal>,
    pub timeline: Vec<TimelineEntry>,
    /// Years that have playtime records, oldest first.
    pub years: Vec<i32>,
}

/// Counts screenshots in the instance folder, including screenshots filed into
/// subfolders such as `2026-11/`.
fn count_screenshots(root: &std::path::Path) -> u64 {
    fn is_image(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "webp"
                )
            })
            .unwrap_or(false)
    }

    fn walk(dir: &std::path::Path, depth: u8) -> u64 {
        if depth > 2 {
            return 0;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return 0;
        };
        let mut total = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else {
                continue;
            };
            if meta.is_dir() {
                total += walk(&path, depth + 1);
            } else if meta.is_file() && is_image(&path) {
                total += 1;
            }
        }
        total
    }

    walk(&root.join("screenshots"), 0)
}

/// Cached result of the expensive part of the page: the whole-library scan.
#[derive(Clone)]
struct CachedOverview {
    fingerprint: String,
    year: i32,
    overview: StatsOverview,
}

static CACHE: std::sync::Mutex<Option<CachedOverview>> = std::sync::Mutex::new(None);

fn cached() -> Option<CachedOverview> {
    CACHE.lock().ok().and_then(|guard| guard.clone())
}

fn store_cache(entry: CachedOverview) {
    if let Ok(mut guard) = CACHE.lock() {
        *guard = Some(entry);
    }
}

fn modified_stamp(path: &std::path::Path) -> u128 {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|elapsed| elapsed.as_millis())
        .unwrap_or(0)
}

/// Newest modification stamp under a world's `saves` folder. Writing a player's
/// advancement file updates the containing folder, not `saves` itself, so the
/// per-world folders and their advancement folders are sampled too.
fn saves_stamp(root: &std::path::Path) -> u128 {
    let saves = root.join("saves");
    let mut newest = modified_stamp(&saves);
    let Ok(entries) = std::fs::read_dir(&saves) else {
        return newest;
    };
    for entry in entries.flatten() {
        let world = entry.path();
        newest = newest.max(modified_stamp(&world));
        newest = newest.max(modified_stamp(&world.join("level.dat")));
        newest = newest.max(modified_stamp(&world.join("players").join("advancements")));
    }
    newest
}

/// Newest modification stamp under a world's `screenshots` folder, including one
/// level of subfolders because the counter walks them too.
fn screenshots_stamp(root: &std::path::Path) -> u128 {
    let shots = root.join("screenshots");
    let mut newest = modified_stamp(&shots);
    let Ok(entries) = std::fs::read_dir(&shots) else {
        return newest;
    };
    for entry in entries.flatten() {
        newest = newest.max(modified_stamp(&entry.path()));
    }
    newest
}

/// Cheap change detector: instance identities, their saves and screenshot
/// folders, game versions and the playtime table. Statting these is far cheaper
/// than parsing every world.
async fn fingerprint(instance_list: &[crate::state::InstanceMetadata]) -> String {
    let mut parts: Vec<String> = vec![format!("n{}", instance_list.len())];
    for meta in instance_list {
        let id = &meta.instance.id;
        match get_full_path(id).await {
            Ok(root) => parts.push(format!(
                "{id}:{}:{}:{}",
                meta.instance.name,
                saves_stamp(&root),
                screenshots_stamp(&root)
            )),
            Err(_) => parts.push(format!("{id}:missing")),
        }
    }
    if let Ok(state) = State::get().await {
        parts.push(format!(
            "v{}",
            modified_stamp(&state.directories.versions_dir())
        ));
        if let Ok(Some(stamp)) = playtime::get_last_modified(&state.pool).await {
            parts.push(format!("p{stamp}"));
        }
    }
    parts.join("|")
}

pub async fn overview(year: Option<i32>, force: bool) -> crate::Result<StatsOverview> {
    let state = State::get().await?;
    let year = year.unwrap_or_else(|| Local::now().year());
    let instance_list = super::instance::list().await.unwrap_or_default();
    let fingerprint = fingerprint(&instance_list).await;

    if !force {
        if let Some(entry) = cached() {
            // Icons arrive from a background download, so a cached result without
            // them is refreshed once they exist.
            let icons_ready = entry
                .overview
                .timeline
                .iter()
                .all(|item| item.icon_path.is_some());
            if entry.year == year && entry.fingerprint == fingerprint && icons_ready {
                return Ok(entry.overview);
            }
        }
    }

    let start = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| crate::ErrorKind::InputError("Invalid year".to_string()))?;
    let end = NaiveDate::from_ymd_opt(year, 12, 31)
        .ok_or_else(|| crate::ErrorKind::InputError("Invalid year".to_string()))?;

    let days = playtime::get_daily_playtime(start, end, &state.pool)
        .await?
        .into_iter()
        .map(|entry| DayPoint {
            date: entry.date,
            seconds: entry.played_seconds,
            top_instance: entry.top_instance_name,
        })
        .collect();

    let total_seconds = playtime::get_total_seconds(&state.pool).await?.max(0) as u64;
    let instances = playtime::get_instance_totals(&state.pool)
        .await?
        .into_iter()
        .map(|(name, seconds)| InstanceTotal {
            name,
            seconds: seconds.max(0) as u64,
        })
        .collect();

    let instance_list = super::instance::list().await.unwrap_or_default();
    let instance_count = instance_list.len() as u64;

    let mut screenshot_count = 0u64;
    // Known achievement ids across every installed version (union, not the
    // biggest catalogue, so completions can never exceed the total).
    let mut known_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    // First unlock per achievement, taken across every instance, world and player.
    let mut first_unlocks: HashMap<String, (String, Option<i64>, String)> = HashMap::new();
    // Title and icon for every achievement the catalogue knows about.
    let mut catalog_meta: HashMap<String, (String, Option<String>)> = HashMap::new();

    for meta in &instance_list {
        let id = meta.instance.id.clone();
        let name = meta.instance.name.clone();

        if let Ok(root) = get_full_path(&id).await {
            screenshot_count += count_screenshots(&root);
        }

        let catalog = advancement_catalog(Some(id.clone())).await.unwrap_or_default();
        for entry in &catalog {
            known_ids.insert(entry.id.clone());
            catalog_meta
                .entry(entry.id.clone())
                .or_insert_with(|| (entry.title.clone(), entry.icon_path.clone()));
        }

        let worlds: Vec<WorldSummary> = list_worlds(&id).await.unwrap_or_default();
        for world in worlds {
            if !world.has_advancements {
                continue;
            }
            let Ok(completed) = world_completed_all_players(&id, &world.name).await else {
                continue;
            };
            for entry in completed {
                if !known_ids.contains(&entry.id) {
                    continue;
                }
                let replace = match first_unlocks.get(&entry.id) {
                    Some((_, recorded, _)) => match (recorded, entry.completed_at) {
                        (Some(current), Some(candidate)) => candidate < *current,
                        (None, Some(_)) => true,
                        _ => false,
                    },
                    None => true,
                };
                if replace {
                    first_unlocks.insert(
                        entry.id.clone(),
                        (world.name.clone(), entry.completed_at, name.clone()),
                    );
                }
            }
        }
    }
    let achievement_total = known_ids.len();

    let mut timeline: Vec<TimelineEntry> = first_unlocks
        .into_iter()
        .map(|(id, (world_name, completed_at, instance_name))| {
            let (title, icon_path) = catalog_meta
                .get(&id)
                .cloned()
                .unwrap_or_else(|| (id.clone(), None));
            TimelineEntry {
                id,
                title,
                icon_path,
                instance_name,
                world_name,
                completed_at,
            }
        })
        .collect();
    timeline.sort_by(|left, right| right.completed_at.cmp(&left.completed_at));
    timeline.truncate(500);

    let mut years: Vec<i32> = Vec::new();
    if let Ok(Some(first)) = playtime::get_first_play_date(&state.pool).await {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(&format!("{first}-01"), "%Y-%m-%d") {
            let current = Local::now().year();
            let start_year = parsed.year().min(current);
            for candidate in start_year..=current {
                years.push(candidate);
            }
        }
    }
    if !years.contains(&year) {
        years.push(year);
        years.sort_unstable();
    }

    let overview = StatsOverview {
        total_seconds,
        instance_count,
        screenshot_count,
        achievement_total: achievement_total as u64,
        days,
        instances,
        timeline,
        years,
    };
    store_cache(CachedOverview {
        fingerprint,
        year,
        overview: overview.clone(),
    });
    Ok(overview)
}
