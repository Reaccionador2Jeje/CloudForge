use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{error::ApiError, instances::AppState};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Image {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub os_family: String,
    pub os_version: String,
    pub architecture: String,
    pub disk_format: String,
    pub artifact_type: String,
    pub source_url: Option<String>,
    pub checksum_sha256: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateImageRequest {
    pub slug: String,
    pub name: String,
    pub os_family: String,
    pub os_version: String,
    pub architecture: String,
    pub disk_format: String,
    #[serde(default)]
    pub artifact_type: Option<String>,
    pub source_url: Option<String>,
    pub checksum_sha256: Option<String>,
}

fn validate_token(value: &str, max_len: usize, allowed: &str) -> bool {
    !value.is_empty()
        && value.len() <= max_len
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || allowed.contains(character))
}

fn validate_image(request: CreateImageRequest) -> Result<CreateImageRequest, ApiError> {
    let slug = request.slug.trim();
    let name = request.name.trim();
    let os_family = request.os_family.trim().to_ascii_lowercase();
    let os_version = request.os_version.trim();
    let architecture = request.architecture.trim();
    let disk_format = request.disk_format.trim().to_ascii_lowercase();
    let artifact_type = request.artifact_type.as_deref().map(str::trim).map(str::to_ascii_lowercase)
        .unwrap_or_else(|| if disk_format == "iso" { "installer_iso".to_owned() } else { "disk_image".to_owned() });

    if !validate_token(slug, 96, "._/-")
        || !slug.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
    {
        return Err(ApiError::Validation("slug must be a catalog identifier of at most 96 characters".to_owned()));
    }
    if name.is_empty() || name.len() > 120 {
        return Err(ApiError::Validation("name must contain between 1 and 120 bytes".to_owned()));
    }
    if !validate_token(&os_family, 32, "-") {
        return Err(ApiError::Validation("os_family must be a lowercase alphanumeric identifier".to_owned()));
    }
    if !validate_token(os_version, 32, ".-_+") {
        return Err(ApiError::Validation("os_version contains unsupported characters".to_owned()));
    }
    if !matches!(architecture, "x86_64" | "aarch64") {
        return Err(ApiError::Validation("architecture must be x86_64 or aarch64".to_owned()));
    }
    if !matches!(disk_format.as_str(), "qcow2" | "raw" | "iso") {
        return Err(ApiError::Validation("disk_format must be qcow2, raw, or iso".to_owned()));
    }
    if !matches!(artifact_type.as_str(), "disk_image" | "installer_iso")
        || (disk_format == "iso") != (artifact_type == "installer_iso")
    {
        return Err(ApiError::Validation("installer_iso artifacts must use ISO format; disk_image artifacts must use qcow2 or raw".to_owned()));
    }

    let source_url = request.source_url.as_deref().map(str::trim).filter(|v| !v.is_empty());
    if source_url.is_some_and(|url| {
        url.len() > 2048 || !url.starts_with("https://") || url.chars().any(char::is_whitespace)
    }) {
        return Err(ApiError::Validation("source_url must be an HTTPS URL without whitespace".to_owned()));
    }

    let checksum = request.checksum_sha256.as_deref().map(str::trim).filter(|v| !v.is_empty());
    if checksum.is_some_and(|value| {
        value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit())
    }) {
        return Err(ApiError::Validation("checksum_sha256 must contain exactly 64 hexadecimal characters".to_owned()));
    }

    Ok(CreateImageRequest {
        slug: slug.to_owned(),
        name: name.to_owned(),
        os_family,
        os_version: os_version.to_owned(),
        architecture: architecture.to_owned(),
        disk_format,
        artifact_type: Some(artifact_type),
        source_url: source_url.map(str::to_owned),
        checksum_sha256: checksum.map(str::to_ascii_lowercase),
    })
}

pub async fn list_images(State(state): State<AppState>) -> Result<Json<Vec<Image>>, ApiError> {
    let pool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let images = sqlx::query_as::<_, Image>(
        "SELECT id, slug, name, os_family, os_version, architecture, disk_format, artifact_type,
                source_url, checksum_sha256, status, created_at
         FROM images ORDER BY name ASC, os_version DESC, architecture ASC",
    )
    .fetch_all(pool)
    .await?;
    Ok(Json(images))
}

pub async fn create_image(
    State(state): State<AppState>,
    Json(request): Json<CreateImageRequest>,
) -> Result<(StatusCode, Json<Image>), ApiError> {
    let request = validate_image(request)?;
    let pool: &PgPool = state.pool.as_ref().ok_or(ApiError::DatabaseNotConfigured)?;
    let image = sqlx::query_as::<_, Image>(
        "INSERT INTO images (slug, name, os_family, os_version, architecture, disk_format, artifact_type, source_url, checksum_sha256)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, slug, name, os_family, os_version, architecture, disk_format, artifact_type,
                   source_url, checksum_sha256, status, created_at",
    )
    .bind(request.slug)
    .bind(request.name.trim())
    .bind(request.os_family)
    .bind(request.os_version)
    .bind(request.architecture)
    .bind(request.disk_format)
    .bind(request.artifact_type)
    .bind(request.source_url)
    .bind(request.checksum_sha256)
    .fetch_one(pool)
    .await?;
    Ok((StatusCode::CREATED, Json(image)))
}

#[cfg(test)]
mod tests {
    use super::{validate_image, CreateImageRequest};

    fn request() -> CreateImageRequest {
        CreateImageRequest {
            slug: "ubuntu/24.04".to_owned(),
            name: "Ubuntu 24.04 LTS".to_owned(),
            os_family: "ubuntu".to_owned(),
            os_version: "24.04".to_owned(),
            architecture: "x86_64".to_owned(),
            disk_format: "qcow2".to_owned(),
            artifact_type: None,
            source_url: Some("https://cloud-images.ubuntu.com/".to_owned()),
            checksum_sha256: Some("a".repeat(64)),
        }
    }

    #[test]
    fn accepts_valid_image_metadata() {
        let image = validate_image(request()).unwrap();
        assert_eq!(image.slug, "ubuntu/24.04");
        assert_eq!(image.architecture, "x86_64");
        assert_eq!(image.checksum_sha256, Some("a".repeat(64)));
    }

    #[test]
    fn rejects_non_https_sources() {
        let mut image = request();
        image.source_url = Some("http://example.com/image".to_owned());
        assert!(validate_image(image).is_err());
    }

    #[test]
    fn rejects_unknown_architecture_and_format() {
        let mut image = request();
        image.architecture = "armv7".to_owned();
        assert!(validate_image(image).is_err());

        let mut image = request();
        image.disk_format = "vdi".to_owned();
        assert!(validate_image(image).is_err());
    }

    #[test]
    fn accepts_windows_server_installer_iso_metadata() {
        let mut image = request();
        image.slug = "windows-server/2025".to_owned();
        image.name = "Windows Server 2025".to_owned();
        image.os_family = "windows-server".to_owned();
        image.os_version = "2025".to_owned();
        image.disk_format = "iso".to_owned();
        image.artifact_type = Some("installer_iso".to_owned());
        image.source_url = None;
        image.checksum_sha256 = None;
        let image = validate_image(image).unwrap();
        assert_eq!(image.artifact_type.as_deref(), Some("installer_iso"));
    }

    #[test]
    fn rejects_mismatched_iso_artifact_type() {
        let mut image = request();
        image.disk_format = "iso".to_owned();
        image.artifact_type = Some("disk_image".to_owned());
        assert!(validate_image(image).is_err());
    }

    #[test]
    fn rejects_invalid_checksums() {
        let mut image = request();
        image.checksum_sha256 = Some("not-a-checksum".to_owned());
        assert!(validate_image(image).is_err());
    }

    #[test]
    fn rejects_path_traversal_in_slugs() {
        let mut image = request();
        image.slug = "../private".to_owned();
        assert!(validate_image(image).is_err());
    }
}
