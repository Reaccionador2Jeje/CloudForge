use serde::Serialize;

use crate::domain_xml::{DomainSpec, DomainSpecError};
use crate::storage::{DiskPlan, DiskPlanError, DiskRequest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmProvisioningRequest {
    pub name: String,
    pub vcpu: u16,
    pub memory_mib: u32,
    pub disk_name: String,
    pub disk_size_gib: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VmProvisioningPlan {
    pub schema_version: u8,
    pub vm_name: String,
    pub disk: DiskPlan,
    pub domain_xml: String,
    pub execution_enabled: bool,
    pub blockers: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VmProvisioningError {
    #[error(transparent)]
    Disk(#[from] DiskPlanError),
    #[error(transparent)]
    Domain(#[from] DomainSpecError),
}

impl VmProvisioningRequest {
    pub fn build_plan(&self) -> Result<VmProvisioningPlan, VmProvisioningError> {
        let disk = DiskRequest {
            name: self.disk_name.clone(),
            size_gib: self.disk_size_gib,
        }
        .build_plan()?;

        let domain = DomainSpec {
            name: self.name.clone(),
            vcpu: self.vcpu,
            memory_mib: self.memory_mib,
            disk_path: disk.managed_path.clone(),
        };

        let domain_xml = domain.render_libvirt_xml()?;

        Ok(VmProvisioningPlan {
            schema_version: 1,
            vm_name: self.name.clone(),
            disk,
            domain_xml,
            execution_enabled: false,
            blockers: vec![
                "storage_provider_not_configured",
                "trusted_node_enrollment_required",
                "scheduler_and_capacity_reservations_required",
                "network_isolation_provider_required",
                "vm_lifecycle_executor_not_implemented",
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{VmProvisioningError, VmProvisioningRequest};

    fn request() -> VmProvisioningRequest {
        VmProvisioningRequest {
            name: "cf-demo".to_owned(),
            vcpu: 2,
            memory_mib: 2048,
            disk_name: "cf-demo-root".to_owned(),
            disk_size_gib: 40,
        }
    }

    #[test]
    fn combines_disk_and_domain_into_one_non_executable_plan() {
        let plan = request().build_plan().unwrap();
        assert_eq!(plan.schema_version, 1);
        assert_eq!(plan.vm_name, "cf-demo");
        assert_eq!(plan.disk.disk_name, "cf-demo-root");
        assert_eq!(
            plan.disk.managed_path,
            "/var/lib/cloudforge/pools/default/cf-demo-root.qcow2"
        );
        assert!(plan.domain_xml.contains(
            "<source file='/var/lib/cloudforge/pools/default/cf-demo-root.qcow2'/>"
        ));
        assert!(!plan.execution_enabled);
        assert!(!plan.blockers.is_empty());
    }

    #[test]
    fn rejects_invalid_disk_before_rendering_domain() {
        let mut input = request();
        input.disk_name = "../escape".to_owned();
        assert!(matches!(
            input.build_plan(),
            Err(VmProvisioningError::Disk(_))
        ));
    }

    #[test]
    fn rejects_invalid_vm_resources() {
        let mut input = request();
        input.vcpu = 0;
        assert!(matches!(
            input.build_plan(),
            Err(VmProvisioningError::Domain(_))
        ));
    }
}
