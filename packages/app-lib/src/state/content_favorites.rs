use chrono::Utc;
use serde::{Deserialize, Serialize};

pub use crate::state::content_types::{
    ContentFavoriteProvider, ContentFavoriteType,
};
use crate::state::store::content_favorites::{
    ContentFavoriteKey, ContentFavoriteRecord, ContentFavorites,
};
use crate::state::store::resource_source_projects::{
    ResourceSourceProjectKey, ResourceSourceProjects,
};
use crate::state::store::{Collection as _, Handle};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentFavorite {
    #[serde(default)]
    pub resource_id: Option<String>,
    pub provider: ContentFavoriteProvider,
    pub project_id: String,
    pub content_type: ContentFavoriteType,
    pub saved_at: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContentFavoriteInput {
    pub provider: ContentFavoriteProvider,
    pub project_id: String,
    pub content_type: ContentFavoriteType,
}

impl ContentFavoriteInput {
    fn validate(&self) -> crate::Result<()> {
        if self.project_id.trim().is_empty() {
            return Err(crate::ErrorKind::InputError(
                "A content favorite must have a project ID".to_string(),
            )
            .into());
        }
        Ok(())
    }
}

pub(crate) async fn list(
    handle: &Handle,
) -> crate::Result<Vec<ContentFavorite>> {
    let mut favorites = Vec::new();
    for (key, record) in ContentFavorites::get_all(handle).await? {
        favorites.push(with_resource(handle, key, record).await?);
    }

    favorites.sort_by(|left, right| {
        right
            .saved_at
            .cmp(&left.saved_at)
            .then_with(|| left.provider.as_str().cmp(right.provider.as_str()))
            .then_with(|| left.project_id.cmp(&right.project_id))
    });
    Ok(favorites)
}

pub(crate) async fn add(
    favorite: ContentFavoriteInput,
    handle: &Handle,
) -> crate::Result<ContentFavorite> {
    favorite.validate()?;
    let key = ContentFavoriteKey {
        provider: favorite.provider,
        project_id: favorite.project_id.trim().to_string(),
    };
    // The write keeps the moment an existing favorite was first saved, so the
    // record the collection hands back is the one the list is ordered by.
    ContentFavorites::set(
        handle,
        &key,
        Some(&ContentFavoriteRecord {
            content_type: favorite.content_type,
            saved_at: Utc::now().timestamp_millis(),
        }),
    )
    .await?;
    let stored = ContentFavorites::get(handle, &key).await?;
    match stored {
        Some(record) => with_resource(handle, key, record).await,
        None => Err(crate::ErrorKind::OtherError(
            "The favorite was removed while it was being saved".to_string(),
        )
        .into()),
    }
}

pub(crate) async fn remove(
    provider: ContentFavoriteProvider,
    project_id: &str,
    handle: &Handle,
) -> crate::Result<()> {
    if project_id.trim().is_empty() {
        return Err(crate::ErrorKind::InputError(
            "A content favorite must have a project ID".to_string(),
        )
        .into());
    }

    let key = ContentFavoriteKey {
        provider,
        project_id: project_id.trim().to_string(),
    };
    ContentFavorites::set(handle, &key, None).await
}

async fn with_resource(
    handle: &Handle,
    key: ContentFavoriteKey,
    record: ContentFavoriteRecord,
) -> crate::Result<ContentFavorite> {
    let resource_id = ResourceSourceProjects::get(
        handle,
        &ResourceSourceProjectKey {
            source: key.provider.as_str().to_string(),
            project_id: key.project_id.clone(),
        },
    )
    .await?
    .map(|project| project.resource_id);

    Ok(ContentFavorite {
        resource_id,
        provider: key.provider,
        project_id: key.project_id,
        content_type: record.content_type,
        saved_at: record.saved_at,
    })
}
