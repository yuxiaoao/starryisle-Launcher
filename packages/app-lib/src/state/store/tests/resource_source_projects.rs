//! What the resource source projects collection promises, and what the
//! favorites read makes of it.

use super::support;
use crate::state::content_favorites::{
    ContentFavoriteInput, ContentFavoriteProvider, ContentFavoriteType, add,
    list,
};
use crate::state::store::Collection as _;
use crate::state::store::Handle;
use crate::state::store::resource_source_projects::{
    ResourceSourceProjectKey, ResourceSourceProjectRecord,
    ResourceSourceProjects,
};

/// A database whose only provider project already maps to a resource.
async fn seeded() -> Handle {
    let pool = support::pool().await;
    support::add_resource(&pool, "resource").await;
    sqlx::query(
        "INSERT INTO resource_source_projects (source, project_id, resource_id)
         VALUES ('modrinth', 'sodium', 'resource')",
    )
    .execute(&pool)
    .await
    .unwrap();
    Handle::new(pool)
}

#[tokio::test]
async fn a_mapping_round_trips_and_removes() {
    let handle = support::handle().await;
    support::add_resource(handle.pool(), "resource").await;
    let project = ResourceSourceProjectKey {
        source: "modrinth".to_string(),
        project_id: "sodium".to_string(),
    };
    let record = ResourceSourceProjectRecord {
        resource_id: "resource".to_string(),
    };

    ResourceSourceProjects::set(&handle, &project, Some(&record))
        .await
        .unwrap();
    assert_eq!(
        ResourceSourceProjects::get(&handle, &project)
            .await
            .unwrap(),
        Some(record)
    );

    ResourceSourceProjects::set(&handle, &project, None)
        .await
        .unwrap();
    assert_eq!(
        ResourceSourceProjects::get(&handle, &project)
            .await
            .unwrap(),
        None
    );
    assert!(
        ResourceSourceProjects::get_all(&handle)
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn a_favorite_carries_the_resource_its_project_maps_to() {
    let handle = seeded().await;

    let favorite = add(
        ContentFavoriteInput {
            provider: ContentFavoriteProvider::Modrinth,
            project_id: "sodium".to_string(),
            content_type: ContentFavoriteType::Mod,
        },
        &handle,
    )
    .await
    .unwrap();

    assert_eq!(favorite.resource_id.as_deref(), Some("resource"));
    assert_eq!(
        list(&handle).await.unwrap()[0].resource_id.as_deref(),
        Some("resource")
    );
}
