use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;

const DEFAULT_POOL: &str = "/var/lib/cloudforge/pools/default";

#[derive(Debug, Clone, Serialize)]
pub struct StoragePreflight {
    pub schema_version: u8,
    pub supported_host: bool,
    pub pool_path: String,
    pub pool_exists: bool,
    pub pool_is_directory: bool,
    pub pool_path_has_symlink: bool,
    pub pool_mode: Option<u32>,
    pub pool_owner_uid: Option<u32>,
    pub pool_has_write_permission: bool,
    pub qemu_img_available: bool,
    pub provider_ready: bool,
    pub blockers: Vec<&'static str>,
}

fn inspect_path(path: &Path) -> (bool, bool, Option<u32>, Option<u32>, bool) {
    use std::os::unix::fs::MetadataExt;

    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return (false, false, None, None, false),
    };
    let is_directory = metadata.is_dir();
    let mode = metadata.mode() & 0o777;
    let writable = mode & 0o222 != 0;
    (
        true,
        is_directory,
        Some(mode),
        Some(metadata.uid()),
        writable,
    )
}

fn path_has_symlink(path: &Path) -> bool {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Ok(metadata) = std::fs::symlink_metadata(&current) {
            if metadata.file_type().is_symlink() {
                return true;
            }
        }
    }
    false
}

fn qemu_img_available() -> bool {
    Command::new("qemu-img")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Read-only inspection of the fixed CloudForge storage pool path.
/// This does not create directories, write probe files, or invoke qemu-img.
pub fn collect_preflight() -> StoragePreflight {
    let supported_host = cfg!(target_os = "linux");

    #[cfg(target_os = "linux")]
    let (pool_exists, pool_is_directory, pool_mode, pool_owner_uid, pool_has_write_permission) =
        inspect_path(Path::new(DEFAULT_POOL));

    #[cfg(not(target_os = "linux"))]
    let (pool_exists, pool_is_directory, pool_mode, pool_owner_uid, pool_has_write_permission) =
        (false, false, None, None, false);

    let pool_path_has_symlink = path_has_symlink(Path::new(DEFAULT_POOL));
    let qemu_img_available = supported_host && qemu_img_available();

    let mut blockers = Vec::new();
    if !supported_host {
        blockers.push("linux_host_required");
    }
    if !pool_exists {
        blockers.push("storage_pool_directory_missing");
    }
    if pool_exists && !pool_is_directory {
        blockers.push("storage_pool_path_not_directory");
    }
    if pool_path_has_symlink {
        blockers.push("storage_pool_path_contains_symlink");
    }
    if pool_is_directory && !pool_has_write_permission {
        blockers.push("storage_pool_has_no_write_mode_bits");
    }
    if !qemu_img_available {
        blockers.push("qemu_img_unavailable");
    }

    let provider_ready = blockers.is_empty();

    StoragePreflight {
        schema_version: 1,
        supported_host,
        pool_path: DEFAULT_POOL.to_owned(),
        pool_exists,
        pool_is_directory,
        pool_path_has_symlink,
        pool_mode,
        pool_owner_uid,
        pool_has_write_permission,
        qemu_img_available,
        provider_ready,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::StoragePreflight;

    #[test]
    fn report_schema_has_explicit_execution_gates() {
        let report = StoragePreflight {
            schema_version: 1,
            supported_host: true,
            pool_path: "/var/lib/cloudforge/pools/default".to_owned(),
            pool_exists: false,
            pool_is_directory: false,
            pool_path_has_symlink: false,
            pool_mode: None,
            pool_owner_uid: None,
            pool_has_write_permission: false,
            qemu_img_available: false,
            provider_ready: false,
            blockers: vec!["storage_pool_directory_missing", "qemu_img_unavailable"],
        };
        assert_eq!(report.schema_version, 1);
        assert!(!report.provider_ready);
        assert!(!report.pool_has_write_permission);
        assert_eq!(report.blockers.len(), 2);
    }
}
