use serde::Serialize;

const DEFAULT_POOL: &str = "/var/lib/cloudforge/pools/default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskRequest {
    pub name: String,
    pub size_gib: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiskPlan {
    pub schema_version: u8,
    pub disk_name: String,
    pub format: &'static str,
    pub virtual_size_bytes: u64,
    pub allocation: &'static str,
    pub managed_path: String,
    pub requires_storage_provider: bool,
    pub execution_enabled: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DiskPlanError {
    #[error("disk name must start with a lowercase ASCII letter and contain at most 48 lowercase letters, digits, hyphens, or underscores")]
    InvalidName,
    #[error("disk size must be between 1 and 2048 GiB")]
    InvalidSize,
}

impl DiskRequest {
    pub fn build_plan(&self) -> Result<DiskPlan, DiskPlanError> {
        if self.name.is_empty()
            || self.name.len() > 48
            || !self.name.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'_')
            })
            || !self.name.as_bytes()[0].is_ascii_lowercase()
        {
            return Err(DiskPlanError::InvalidName);
        }

        if !(1..=2048).contains(&self.size_gib) {
            return Err(DiskPlanError::InvalidSize);
        }

        let virtual_size_bytes = u64::from(self.size_gib) * 1024 * 1024 * 1024;
        Ok(DiskPlan {
            schema_version: 1,
            disk_name: self.name.clone(),
            format: "qcow2",
            virtual_size_bytes,
            allocation: "sparse",
            managed_path: format!("{DEFAULT_POOL}/{}.qcow2", self.name),
            requires_storage_provider: true,
            execution_enabled: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DiskPlanError, DiskRequest};

    #[test]
    fn creates_a_deterministic_sparse_qcow2_plan() {
        let plan = DiskRequest {
            name: "root-disk_01".to_owned(),
            size_gib: 40,
        }
        .build_plan()
        .unwrap();

        assert_eq!(plan.schema_version, 1);
        assert_eq!(plan.format, "qcow2");
        assert_eq!(plan.virtual_size_bytes, 42_949_672_960);
        assert_eq!(plan.allocation, "sparse");
        assert_eq!(
            plan.managed_path,
            "/var/lib/cloudforge/pools/default/root-disk_01.qcow2"
        );
        assert!(plan.requires_storage_provider);
        assert!(!plan.execution_enabled);
    }

    #[test]
    fn rejects_unsafe_disk_names() {
        for name in ["../escape", "UpperCase", "disk/name", "disk name", ""] {
            let request = DiskRequest {
                name: name.to_owned(),
                size_gib: 10,
            };
            assert_eq!(request.build_plan(), Err(DiskPlanError::InvalidName));
        }
    }

    #[test]
    fn enforces_disk_size_limits() {
        for size_gib in [0, 2049] {
            let request = DiskRequest {
                name: "root".to_owned(),
                size_gib,
            };
            assert_eq!(request.build_plan(), Err(DiskPlanError::InvalidSize));
        }
    }
}
