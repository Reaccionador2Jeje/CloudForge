use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json as SqlJson, FromRow};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedUser,
    error::ApiError,
    instances::AppState,
};

#[derive(Debug, Serialize, FromRow)]
pub struct InstallationJob {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub profile_id: String,
    pub installer_image_id: Uuid,
    pub status: String,
    pub execution_enabled: bool,
    pub blockers: SqlJson<Vec<String>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInstallationRequest {
    pub instance_id: Uuid,
    pub profile_id: String,
    pub installer_image_slug: String,
}

#[derive(Debug, FromRow)]
struct InstallerImage {
    id: Uuid,
    os_family: String,
    os_version: String,
    architecture: String,
    artifact_type: String,
    disk_format: String,
    status: String,
    source_url: Option<String>,
    checksum_sha256: Option<String>,
}

fn profile_version(profile_id: &str) -> Option<&'static str> {
    match profile_id {
        "windows-server-2025" => Some("2025"),
        "windows-server-2022" => Some("2022"),
        _ => None,
    }
}

fn installation_blockers(image: &InstallerImage) -> Vec<String> {
    let mut blockers = vec![
        "qemu_kvm_provider_unavailable".to_owned(),
        "virtual_hardware_compatibility_not_validated".to_owned(),
        "virtio_driver_delivery_not_implemented".to_owned(),
        "windows_server_licensing_workflow_not_configured".to_owned(),
    ];

    if image.status != "verified"
        || image.source_url.is_none()
        || image.checksum_sha256.is_none()
    {
        blockers.push("installer_media_not_verified".to_owned());
    }

    blockers
}

pub async fn list_installations(
    State(state): State<AppState>,
) -> Result<Json<Vec<InstallationJob>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let jobs = sqlx::query_as::<_, InstallationJob>(
        "SELECT id, instance_id, profile_id, installer_image_id, status,
                execution_enabled, blockers, created_by, created_at, updated_at
         FROM installation_jobs
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(Json(jobs))
}

pub async fn get_installation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<InstallationJob>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let job = sqlx::query_as::<_, InstallationJob>(
        "SELECT id, instance_id, profile_id, installer_image_id, status,
                execution_enabled, blockers, created_by, created_at, updated_at
         FROM installation_jobs
         WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(ApiError::NotFound("installation job was not found".to_owned()))?;

    Ok(Json(job))
}

pub async fn create_installation(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreateInstallationRequest>,
) -> Result<(StatusCode, Json<InstallationJob>), ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let version = profile_version(request.profile_id.trim()).ok_or_else(|| {
        ApiError::Validation(
            "profile_id must be windows-server-2025 or windows-server-2022".to_owned(),
        )
    })?;
    let slug = request.installer_image_slug.trim();
    if slug.is_empty() || slug.len() > 96 {
        return Err(ApiError::Validation(
            "installer_image_slug must contain between 1 and 96 bytes".to_owned(),
        ));
    }

    let instance_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM instances WHERE id = $1)")
            .bind(request.instance_id)
            .fetch_one(pool)
            .await?;
    if !instance_exists {
        return Err(ApiError::Validation(
            "instance_id does not identify an existing instance".to_owned(),
        ));
    }

    let image = sqlx::query_as::<_, InstallerImage>(
        "SELECT id, os_family, os_version, architecture, artifact_type,
                disk_format, status, source_url, checksum_sha256
         FROM images WHERE slug = $1",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        ApiError::Validation("installer image was not found in the image catalog".to_owned())
    })?;

    if image.os_family != "windows-server"
        || image.os_version != version
        || image.architecture != "x86_64"
        || image.artifact_type != "installer_iso"
        || image.disk_format != "iso"
    {
        return Err(ApiError::Validation(
            "installer image does not match the selected Windows Server profile".to_owned(),
        ));
    }

    let blockers = installation_blockers(&image);
    let job = sqlx::query_as::<_, InstallationJob>(
        "INSERT INTO installation_jobs
            (instance_id, profile_id, installer_image_id, blockers, created_by)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, instance_id, profile_id, installer_image_id, status,
                   execution_enabled, blockers, created_by, created_at, updated_at",
    )
    .bind(request.instance_id)
    .bind(request.profile_id.trim())
    .bind(image.id)
    .bind(SqlJson(blockers))
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    sqlx::query(
        "INSERT INTO audit_events
            (actor_user_id, event_type, resource_type, resource_id, outcome, details)
         VALUES ($1, 'compute.installation.planned', 'installation_job', $2, 'success',
                 jsonb_build_object('profile_id', $3, 'installer_image_slug', $4))",
    )
    .bind(user.id)
    .bind(job.id.to_string())
    .bind(request.profile_id.trim())
    .bind(slug)
    .execute(pool)
    .await?;

    Ok((StatusCode::CREATED, Json(job)))
}

#[cfg(test)]
mod tests {
    use super::{installation_blockers, profile_version, InstallerImage};
    use uuid::Uuid;

    fn draft_image() -> InstallerImage {
        InstallerImage {
            id: Uuid::nil(),
            os_family: "windows-server".to_owned(),
            os_version: "2025".to_owned(),
            architecture: "x86_64".to_owned(),
            artifact_type: "installer_iso".to_owned(),
            disk_format: "iso".to_owned(),
            status: "draft".to_owned(),
            source_url: None,
            checksum_sha256: None,
        }
    }

    #[test]
    fn only_supported_windows_profiles_are_accepted() {
        assert_eq!(profile_version("windows-server-2025"), Some("2025"));
        assert_eq!(profile_version("windows-server-2022"), Some("2022"));
        assert_eq!(profile_version("ubuntu-24-04"), None);
    }

    #[test]
    fn planned_jobs_always_expose_execution_blockers() {
        let blockers = installation_blockers(&draft_image());
        assert!(blockers.contains(&"qemu_kvm_provider_unavailable".to_owned()));
        assert!(blockers.contains(&"installer_media_not_verified".to_owned()));
        assert!(blockers.contains(
            &"windows_server_licensing_workflow_not_configured".to_owned()
        ));
    }
}
