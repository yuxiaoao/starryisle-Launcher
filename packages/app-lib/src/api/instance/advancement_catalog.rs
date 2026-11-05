// Builds the advancement catalogue from the player's own game jar.
//
// Nothing from Mojang ships with the launcher: the titles, descriptions and item
// textures are read out of the jar the player already installed, then cached next
// to the launcher's other caches.

use super::advancements::version_jar;
use crate::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancementDef {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub icon_item: Option<String>,
    pub icon_path: Option<String>,
    pub hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedCatalog {
    version: u32,
    jar_mtime: u64,
    jar_size: u64,
    entries: Vec<AdvancementDef>,
}

/// Bumped whenever the extraction logic changes, so old caches are rebuilt.
const CACHE_VERSION: u32 = 5;

/// Icons that live under an entity texture rather than an item texture.
fn entity_icon(item: &str) -> Option<&'static str> {
    match item {
        "dragon_head" => Some("entity/enderdragon/dragon.png"),
        "wither_skeleton_skull" => Some("entity/wither/wither.png"),
        _ => None,
    }
}

fn read_entry(archive: &mut ZipArchive<std::fs::File>, name: &str) -> Option<String> {
    let mut file = archive.by_name(name).ok()?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).ok()?;
    Some(buffer)
}

fn read_bytes(archive: &mut ZipArchive<std::fs::File>, name: &str) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;
    Some(buffer)
}

/// Reads a file out of the launcher's asset store, which keeps the game assets
/// that are not packed inside the version jar.
fn read_asset(assets: &Path, objects: &HashMap<String, String>, key: &str) -> Option<Vec<u8>> {
    let hash = objects.get(key)?;
    if hash.len() < 2 {
        return None;
    }
    let path = assets.join("objects").join(&hash[..2]).join(hash);
    std::fs::read(path).ok()
}

fn asset_index(assets: &Path) -> HashMap<String, String> {
    let mut merged = HashMap::new();
    let Ok(entries) = std::fs::read_dir(assets.join("indexes")) else {
        return merged;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        if let Some(objects) = value.get("objects").and_then(|inner| inner.as_object()) {
            for (key, meta) in objects {
                if let Some(hash) = meta.get("hash").and_then(|inner| inner.as_str()) {
                    merged.insert(key.clone(), hash.to_string());
                }
            }
        }
    }
    merged
}

fn load_lang(
    archive: &mut ZipArchive<std::fs::File>,
    assets: &Path,
    objects: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = HashMap::new();
    for name in [
        "assets/minecraft/lang/en_us.json",
        "assets/minecraft/lang/zh_cn.json",
    ] {
        if let Some(text) = read_entry(archive, name) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&text) {
                merged.extend(map);
            }
        }
    }
    // Newer installs keep the language files in the asset store instead of the jar.
    if !merged.contains_key("advancements.story.root.title") {
        if let Some(bytes) = read_asset(assets, objects, "minecraft/lang/en_us.json") {
            if let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&bytes) {
                merged.extend(map);
            }
        }
    }
    if let Some(bytes) = read_asset(assets, objects, "minecraft/lang/zh_cn.json") {
        if let Ok(map) = serde_json::from_slice::<HashMap<String, String>>(&bytes) {
            merged.extend(map);
        }
    }
    merged
}

fn resolve_text(value: &serde_json::Value, lang: &HashMap<String, String>) -> String {
    if let Some(text) = value.get("text").and_then(|inner| inner.as_str()) {
        return text.to_string();
    }
    if let Some(key) = value.get("translate").and_then(|inner| inner.as_str()) {
        return lang.get(key).cloned().unwrap_or_else(|| key.to_string());
    }
    String::new()
}

fn category_of(id: &str) -> String {
    let path = id.strip_prefix("minecraft:").unwrap_or(id);
    match path.split('/').next() {
        Some("story") => "story".to_string(),
        Some("nether") => "nether".to_string(),
        Some("end") => "end".to_string(),
        Some("adventure") => "adventure".to_string(),
        Some("husbandry") => "husbandry".to_string(),
        _ => "other".to_string(),
    }
}

fn icon_item_id(value: &serde_json::Value) -> Option<String> {
    // Older versions use "item"; 1.21.5+ moved the id under "id".
    let raw = value
        .get("item")
        .and_then(|inner| inner.as_str())
        .or_else(|| value.get("id").and_then(|inner| inner.as_str()))?;
    let trimmed = raw.strip_prefix("minecraft:").unwrap_or(raw);
    Some(trimmed.to_string())
}

/// Downloads the 32x32 icon renders from the Chinese Minecraft Wiki and caches
/// them on disk. The wiki names each row after the advancement id, so the icons
/// can be matched without any translation step.
async fn download_wiki_icons(cache_root: &Path, ids: &[String]) {
    const CACHE_TTL_SECS: u64 = 7 * 24 * 60 * 60;
    let dir = cache_root.join("wiki-icons");
    let marker = dir.join("fetched.json");


    // Reuse the previous download while it is still fresh.
    let fresh = std::fs::metadata(&marker)
        .ok()
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.elapsed().ok())
        .map(|age| age.as_secs() < CACHE_TTL_SECS)
        .unwrap_or(false);
    if fresh {
        return;
    }

    let Some(page) =
        fetch_text("https://zh.minecraft.wiki/w/%E8%BF%9B%E5%BA%A6?variant=zh-cn").await
    else {
        return;
    };
    let Ok(row_re) = regex::Regex::new(r"(?s)<tr[^>]*>(.*?)</tr>") else {
        return;
    };
    let Ok(icon_re) = regex::Regex::new(r"Invicon_([A-Za-z0-9_-]+)\.(?:png|gif)") else {
        return;
    };
    let Ok(id_re) = regex::Regex::new(r"<code>([a-z0-9_]+)<wbr[^>]*>/?([a-z0-9_/]*)</code>") else {
        return;
    };

    // Every advancement is one table row: the first icon in the row is its
    // picture, and the code cell carries its namespace id.
    let mut rows: Vec<(String, String)> = Vec::new();
    for row in row_re.captures_iter(&page) {
        let Some(body) = row.get(1) else { continue; };
        let body = body.as_str();
        let Some(icon_name) = icon_re.captures(body).and_then(|capture| capture.get(1)) else {
            continue;
        };
        let Some(id_match) = id_re.captures(body) else {
            continue;
        };
        let (Some(namespace), Some(path)) = (id_match.get(1), id_match.get(2)) else {
            continue;
        };
        let path = path.as_str().trim_start_matches('/');
        if path.is_empty() {
            continue;
        }
        rows.push((
            format!("minecraft:{}/{}", namespace.as_str(), path),
            icon_name.as_str().to_string(),
        ));
    }
    let Ok(_) = std::fs::create_dir_all(&dir) else {
        return;
    };
    let mut tasks = Vec::new();
    for (id, name) in rows {
        if !ids.iter().any(|known| known == &id) {
            continue;
        }
        let dir = dir.clone();
        tasks.push(tokio::spawn(async move {
            let url = format!("https://zh.minecraft.wiki/images/Invicon_{name}.png");
            let bytes = fetch_bytes(&url).await?;
            let file = dir.join(format!("{}.png", slug_id(&id)));
            std::fs::write(&file, bytes).ok()?;
            Some(())
        }));
    }
    let mut downloaded = 0usize;
    for task in tasks {
        if task.await.ok().flatten().is_some() {
            downloaded += 1;
        }
    }
    if downloaded > 0 {
        let _ = std::fs::write(&marker, b"{}");
    }
}

fn slug_id(id: &str) -> String {
    id.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

async fn fetch_text(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(25))
        .user_agent("StarryIsleLauncher/1.0 (advancement icons)")
        .build()
        .ok()?;
    let response = client.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.text().await.ok()
}

async fn fetch_bytes(url: &str) -> Option<Vec<u8>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("StarryIsleLauncher/1.0 (advancement icons)")
        .build()
        .ok()?;
    let response = client.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.bytes().await.ok().map(|bytes| bytes.to_vec())
}

/// Points each achievement at its wiki icon. Existing files are applied right
/// away; a missing set is downloaded in the background so opening the page never
/// waits on the network.
async fn apply_wiki_icons(entries: &mut [AdvancementDef], cache_root: &Path) {
    let dir = cache_root.join("wiki-icons");
    let mut applied = 0;
    for entry in entries.iter_mut() {
        let file = dir.join(format!("{}.png", slug_id(&entry.id)));
        if file.is_file() {
            entry.icon_path = Some(file.to_string_lossy().to_string());
            applied += 1;
        }
    }
    if applied > 0 {
        return;
    }

    let ids: Vec<String> = entries.iter().map(|entry| entry.id.clone()).collect();
    let root = cache_root.to_path_buf();
    tokio::spawn(async move {
        download_wiki_icons(&root, &ids).await;
    });
}pub async fn advancement_catalog(
    instance_id: Option<String>,
) -> crate::Result<Vec<AdvancementDef>> {
    let state = State::get().await?;
    let Some(jar) = version_jar(instance_id.as_deref()).await? else {
        return Ok(Vec::new());
    };

    let meta = std::fs::metadata(&jar)?;
    let jar_size = meta.len();
    let jar_mtime = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_secs())
        .unwrap_or(0);

    let cache_root = state.directories.caches_dir().join("advancement-cache");
    let icon_root = cache_root.join("icons");
    let catalog_file = cache_root.join("catalog.json");
    std::fs::create_dir_all(&icon_root)?;

    if let Ok(text) = std::fs::read_to_string(&catalog_file) {
        if let Ok(cached) = serde_json::from_str::<CachedCatalog>(&text) {
            if cached.version == CACHE_VERSION
                && cached.jar_mtime == jar_mtime
                && cached.jar_size == jar_size
            {
                let mut entries = cached.entries;
                apply_wiki_icons(&mut entries, &cache_root).await;
                return Ok(entries);
            }
        }
    }

    let assets_dir = state.directories.assets_dir();
    let entries = tokio::task::spawn_blocking({
        let icon_root = icon_root.clone();
        let jar = jar.clone();
        let assets_dir = assets_dir.clone();
        move || -> crate::Result<Vec<AdvancementDef>> {
            let file = std::fs::File::open(&jar)?;
            let mut archive =
                ZipArchive::new(file).map_err(std::io::Error::from)?;
            let objects = asset_index(&assets_dir);
            let lang = load_lang(&mut archive, &assets_dir, &objects);

            let mut names = Vec::new();
            let mut textures: Vec<String> = Vec::new();
            for index in 0..archive.len() {
                let Ok(entry) = archive.by_index(index) else {
                    continue;
                };
                let Ok(raw_name) = entry.name() else {
                    continue;
                };
                let name = raw_name.to_string();
                if name.starts_with("assets/minecraft/textures/") && name.ends_with(".png") {
                    textures.push(name.clone());
                }
                if !name.starts_with("data/minecraft/advancement") {
                    continue;
                }
                if !name.ends_with(".json") {
                    continue;
                }
                names.push(name);
            }

            let mut defs = Vec::new();
            for name in names {
                let Some(text) = read_entry(&mut archive, &name) else {
                    continue;
                };
                let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
                    continue;
                };
                let Some(display) = value.get("display") else {
                    // Root entries without a display are not shown in game either.
                    continue;
                };

                let stem = name
                    .trim_start_matches("data/minecraft/advancement")
                    .trim_end_matches(".json")
                    .trim_start_matches('/')
                    .to_string();
                let id = format!("minecraft:{stem}");

                let title = display
                    .get("title")
                    .map(|value| resolve_text(value, &lang))
                    .filter(|text| !text.is_empty())
                    .unwrap_or_else(|| stem.clone());
                let description = display
                    .get("description")
                    .map(|value| resolve_text(value, &lang))
                    .unwrap_or_default();
                let icon_item = display.get("icon").and_then(icon_item_id);
                let hidden = display
                    .get("hidden")
                    .and_then(|value| value.as_bool())
                    .unwrap_or(false);


                let mut icon_path = None;
                if let Some(item) = &icon_item {
                    let file_name = format!("{item}.png");
                    let target = icon_root.join(&file_name);
                    if !target.exists() {
                        let mut written = false;

                        // Exact item or block texture, first from the jar then from assets.
                            for folder in ["item", "block"] {
                            let source =
                                format!("assets/minecraft/textures/{folder}/{file_name}");
                            if let Some(bytes) = read_bytes(&mut archive, &source) {
                                std::fs::write(&target, bytes)?;
                                written = true;
                                break;
                            }
                            let key = format!("minecraft/textures/{folder}/{file_name}");
                            if let Some(bytes) = read_asset(&assets_dir, &objects, &key) {
                                std::fs::write(&target, bytes)?;
                                written = true;
                                break;
                            }
                        }

                        // Blocks and a few items keep their art under a suffixed name,
                        // for example `crossbow_standby` or `grass_block_top`.
                        if !written {
                            'suffixes: for suffix in
                                ["_standby", "_top", "_side", "_front", "_still"]
                            {
                                for folder in ["item", "block"] {
                                    let source = format!(
                                        "assets/minecraft/textures/{folder}/{item}{suffix}.png"
                                    );
                                    if let Some(bytes) = read_bytes(&mut archive, &source) {
                                        std::fs::write(&target, bytes)?;
                                        written = true;
                                        break 'suffixes;
                                    }
                                    let key =
                                        format!("minecraft/textures/{folder}/{item}{suffix}.png");
                                    if let Some(bytes) = read_asset(&assets_dir, &objects, &key) {
                                        std::fs::write(&target, bytes)?;
                                        written = true;
                                        break 'suffixes;
                                    }
                                }
                            }
                        }

                        // Any texture whose file name starts with the item id.
                        if !written {
                            if let Some(found) = textures.iter().find(|candidate| {
                                candidate
                                    .rsplit('/')
                                    .next()
                                    .map(|file| {
                                        file.trim_end_matches(".png").starts_with(item.as_str())
                                    })
                                    .unwrap_or(false)
                            }) {
                                if let Some(bytes) = read_bytes(&mut archive, found) {
                                    std::fs::write(&target, bytes)?;
                                    written = true;
                                }
                            }
                        }

                        // Heads and skulls borrow an entity texture.
                        if !written {
                            if let Some(path) = entity_icon(item) {
                                let source = format!("assets/minecraft/textures/{path}");
                                if let Some(bytes) = read_bytes(&mut archive, &source) {
                                    std::fs::write(&target, bytes)?;
                                    written = true;
                                }
                            }
                        }

                        // Resolve the item model and use its layer0 texture.
                        if !written {
                            for folder in ["item", "block"] {
                                let model =
                                    format!("assets/minecraft/models/{folder}/{item}.json");
                                let Some(text) = read_entry(&mut archive, &model) else {
                                    continue;
                                };
                                let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
                                else {
                                    continue;
                                };
                                let texture = value
                                    .get("textures")
                                    .and_then(|inner| inner.get("layer0"))
                                    .and_then(|inner| inner.as_str())
                                    .unwrap_or("");
                                let path = texture.strip_prefix("minecraft:").unwrap_or(texture);
                                if path.is_empty() {
                                    continue;
                                }
                                let source = format!("assets/minecraft/textures/{path}.png");
                                if let Some(bytes) = read_bytes(&mut archive, &source) {
                                    std::fs::write(&target, bytes)?;
                                    written = true;
                                    break;
                                }
                                let key = format!("minecraft/textures/{path}.png");
                                if let Some(bytes) = read_asset(&assets_dir, &objects, &key) {
                                    std::fs::write(&target, bytes)?;
                                    written = true;
                                    break;
                                }
                            }
                        }
                    }
                    if target.exists() {
                        icon_path = Some(target.to_string_lossy().to_string());
                    }
                }

                defs.push(AdvancementDef {
                    id,
                    title,
                    description,
                    category: category_of(&stem),
                    icon_item,
                    icon_path,
                    hidden,
                });
            }

            defs.sort_by(|left, right| left.id.cmp(&right.id));
            Ok(defs)
        }
    })
    .await??;

    let cached = CachedCatalog {
        version: CACHE_VERSION,
        jar_mtime,
        jar_size,
        entries: entries.clone(),
    };
    if let Ok(text) = serde_json::to_string(&cached) {
        let _ = std::fs::write(&catalog_file, text);
    }

    // Wiki icons may already be on disk from an earlier run; apply them now and
    // persist the paths so later starts read them straight from the cache.
    let mut entries = entries;
    apply_wiki_icons(&mut entries, &cache_root).await;
    if entries.iter().any(|entry| entry.icon_path.is_some()) {
        let patched = CachedCatalog {
            version: CACHE_VERSION,
            jar_mtime,
            jar_size,
            entries: entries.clone(),
        };
        if let Ok(text) = serde_json::to_string(&patched) {
            let _ = std::fs::write(&catalog_file, text);
        }
    }

    Ok(entries)
}
