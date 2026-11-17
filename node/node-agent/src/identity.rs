use std::{env, fs, path::PathBuf};

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct IdentityConfig {
    pub node_id: Uuid,
    pub control_plane_url: String,
    pub client_certificate_path: PathBuf,
    pub client_private_key_path: PathBuf,
    pub server_ca_certificate_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum IdentityConfigError {
    #[error("missing required environment variable: {0}")]
    MissingVariable(&'static str),
    #[error("node identity is not a valid UUID")]
    InvalidNodeId,
    #[error("control plane URL must use HTTPS and include a host")]
    InvalidControlPlaneUrl,
    #[error("identity file is unavailable or invalid: {0}")]
    InvalidIdentityFile(&'static str),
    #[error("node private key permissions are too broad; use owner-only access")]
    InsecurePrivateKeyPermissions,
}

impl IdentityConfig {
    pub fn from_env() -> Result<Self, IdentityConfigError> {
        let node_id = required("CLOUDFORGE_NODE_ID")?
            .parse()
            .map_err(|_| IdentityConfigError::InvalidNodeId)?;
        let control_plane_url = required("CLOUDFORGE_CONTROL_PLANE_URL")?;
        validate_https_url(&control_plane_url)?;

        let config = Self {
            node_id,
            control_plane_url,
            client_certificate_path: PathBuf::from(required("CLOUDFORGE_NODE_CLIENT_CERT")?),
            client_private_key_path: PathBuf::from(required("CLOUDFORGE_NODE_CLIENT_KEY")?),
            server_ca_certificate_path: PathBuf::from(required("CLOUDFORGE_NODE_CA_CERT")?),
        };

        config.validate_files()?;
        Ok(config)
    }

    fn validate_files(&self) -> Result<(), IdentityConfigError> {
        let certificate = read_file(&self.client_certificate_path, "client certificate")?;
        let private_key = read_file(&self.client_private_key_path, "client private key")?;
        let ca_certificate = read_file(&self.server_ca_certificate_path, "server CA certificate")?;

        if !certificate.contains("-----BEGIN CERTIFICATE-----")
            || !(private_key.contains("-----BEGIN PRIVATE KEY-----")
                || private_key.contains("-----BEGIN RSA PRIVATE KEY-----")
                || private_key.contains("-----BEGIN EC PRIVATE KEY-----"))
            || !ca_certificate.contains("-----BEGIN CERTIFICATE-----")
        {
            return Err(IdentityConfigError::InvalidIdentityFile("PEM content"));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&self.client_private_key_path)
                .map_err(|_| IdentityConfigError::InvalidIdentityFile("client private key"))?;
            if metadata.permissions().mode() & 0o077 != 0 {
                return Err(IdentityConfigError::InsecurePrivateKeyPermissions);
            }
        }

        Ok(())
    }

    pub fn safe_summary(&self) -> IdentitySummary {
        IdentitySummary {
            node_id: self.node_id,
            control_plane_url: self.control_plane_url.clone(),
            authentication: "mutual_tls",
            credentials_validated: true,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct IdentitySummary {
    pub node_id: Uuid,
    pub control_plane_url: String,
    pub authentication: &'static str,
    pub credentials_validated: bool,
}

fn required(name: &'static str) -> Result<String, IdentityConfigError> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or(IdentityConfigError::MissingVariable(name))
}

fn read_file(path: &PathBuf, label: &'static str) -> Result<String, IdentityConfigError> {
    fs::read_to_string(path).map_err(|_| IdentityConfigError::InvalidIdentityFile(label))
}

fn validate_https_url(value: &str) -> Result<(), IdentityConfigError> {
    let Some((scheme, remainder)) = value.split_once("://") else {
        return Err(IdentityConfigError::InvalidControlPlaneUrl);
    };
    let authority = remainder.split('/').next().unwrap_or_default();
    if scheme != "https" || authority.is_empty() || authority.contains('@') {
        return Err(IdentityConfigError::InvalidControlPlaneUrl);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_https_url, IdentityConfigError};

    #[test]
    fn accepts_https_control_plane_url() {
        assert!(validate_https_url("https://control.example.test").is_ok());
    }

    #[test]
    fn rejects_plain_http_control_plane_url() {
        assert!(matches!(
            validate_https_url("http://control.example.test"),
            Err(IdentityConfigError::InvalidControlPlaneUrl)
        ));
    }

    #[test]
    fn rejects_embedded_url_credentials() {
        assert!(validate_https_url("https://user:secret@control.example.test").is_err());
    }
}
