use serde::Serialize;
use uuid::Uuid;

const DEFAULT_RESERVE_GIB: u64 = 10;
const MAX_DISK_GIB: u64 = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageAllocationRequest {
    pub project_id: String,
    pub quota_gib: u64,
    pub used_gib: u64,
    pub reserved_gib: u64,
    pub requested_gib: u64,
    pub node_available_gib: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StorageAllocationPlan {
    pub schema_version: u8,
    pub project_id: String,
    pub quota_gib: u64,
    pub used_gib: u64,
    pub reserved_gib: u64,
    pub requested_gib: u64,
    pub projected_usage_gib: u64,
    pub quota_remaining_after_gib: u64,
    pub node_available_gib: u64,
    pub required_node_capacity_gib: u64,
    pub execution_enabled: bool,
    pub reservation_created: bool,
    pub blockers: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StorageAllocationError {
    #[error("project_id must be a valid UUID")]
    InvalidProjectId,
    #[error("quota must be between 1 and 1,048,576 GiB")]
    InvalidQuota,
    #[error("requested disk size must be between 1 and 2048 GiB")]
    InvalidRequestSize,
}

impl StorageAllocationRequest {
    pub fn build_plan(&self) -> Result<StorageAllocationPlan, StorageAllocationError> {
        let project_id = Uuid::parse_str(&self.project_id)
            .map_err(|_| StorageAllocationError::InvalidProjectId)?
            .to_string();

        if !(1..=1_048_576).contains(&self.quota_gib) {
            return Err(StorageAllocationError::InvalidQuota);
        }
        if !(1..=MAX_DISK_GIB).contains(&self.requested_gib) {
            return Err(StorageAllocationError::InvalidRequestSize);
        }

        let projected_usage_gib = self
            .used_gib
            .saturating_add(self.reserved_gib)
            .saturating_add(self.requested_gib);
        let quota_remaining_after_gib = self.quota_gib.saturating_sub(projected_usage_gib);
        let required_node_capacity_gib = self.requested_gib.saturating_add(DEFAULT_RESERVE_GIB);

        let mut blockers = Vec::new();
        if projected_usage_gib > self.quota_gib {
            blockers.push("project_storage_quota_exceeded");
        }
        if self.node_available_gib < required_node_capacity_gib {
            blockers.push("insufficient_node_capacity");
        }

        Ok(StorageAllocationPlan {
            schema_version: 1,
            project_id,
            quota_gib: self.quota_gib,
            used_gib: self.used_gib,
            reserved_gib: self.reserved_gib,
            requested_gib: self.requested_gib,
            projected_usage_gib,
            quota_remaining_after_gib,
            node_available_gib: self.node_available_gib,
            required_node_capacity_gib,
            execution_enabled: false,
            reservation_created: false,
            blockers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{StorageAllocationError, StorageAllocationRequest};

    fn request() -> StorageAllocationRequest {
        StorageAllocationRequest {
            project_id: "a1b2c3d4-e5f6-47a8-9012-123456789abc".to_owned(),
            quota_gib: 200,
            used_gib: 50,
            reserved_gib: 20,
            requested_gib: 40,
            node_available_gib: 100,
        }
    }

    #[test]
    fn creates_valid_non_executable_allocation_plan() {
        let plan = request().build_plan().unwrap();
        assert_eq!(plan.projected_usage_gib, 110);
        assert_eq!(plan.quota_remaining_after_gib, 90);
        assert_eq!(plan.required_node_capacity_gib, 50);
        assert!(plan.blockers.is_empty());
        assert!(!plan.execution_enabled);
        assert!(!plan.reservation_created);
    }

    #[test]
    fn blocks_project_quota_overcommit() {
        let mut input = request();
        input.quota_gib = 100;
        let plan = input.build_plan().unwrap();
        assert!(plan.blockers.contains(&"project_storage_quota_exceeded"));
    }

    #[test]
    fn blocks_insufficient_node_capacity() {
        let mut input = request();
        input.node_available_gib = 49;
        let plan = input.build_plan().unwrap();
        assert!(plan.blockers.contains(&"insufficient_node_capacity"));
    }

    #[test]
    fn rejects_invalid_project_and_request() {
        let mut input = request();
        input.project_id = "not-a-uuid".to_owned();
        assert_eq!(
            input.build_plan(),
            Err(StorageAllocationError::InvalidProjectId)
        );

        let mut input = request();
        input.requested_gib = 2049;
        assert_eq!(
            input.build_plan(),
            Err(StorageAllocationError::InvalidRequestSize)
        );
    }

    #[test]
    fn saturating_arithmetic_cannot_wrap_into_available_quota() {
        let mut input = request();
        input.used_gib = u64::MAX;
        let plan = input.build_plan().unwrap();
        assert!(plan.blockers.contains(&"project_storage_quota_exceeded"));
        assert_eq!(plan.projected_usage_gib, u64::MAX);
    }
}
