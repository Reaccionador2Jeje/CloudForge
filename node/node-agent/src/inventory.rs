use serde::Serialize;
use sysinfo::{Disks, System};

/// Read-only snapshot of host capabilities. Values are informational,
/// not a reservation or guarantee of allocatable capacity.
#[derive(Debug, Clone, Serialize)]
pub struct HostInventory {
    pub schema_version: u32,
    pub hostname: Option<String>,
    pub operating_system: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub architecture: Option<String>,
    pub cpu_logical_cores: usize,
    pub memory_total_bytes: u64,
    pub memory_available_bytes: u64,
    pub disks: Vec<DiskInventory>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInventory {
    pub name: String,
    pub filesystem: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

impl HostInventory {
    pub fn collect() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        let disks = Disks::new_with_refreshed_list()
            .iter()
            .map(|disk| DiskInventory {
                name: disk.name().to_string_lossy().into_owned(),
                filesystem: disk.file_system().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
            })
            .collect();

        Self {
            schema_version: 1,
            hostname: System::host_name(),
            operating_system: System::name(),
            os_version: System::os_version(),
            kernel_version: System::kernel_version(),
            architecture: System::cpu_arch(),
            cpu_logical_cores: system.cpus().len(),
            memory_total_bytes: system.total_memory(),
            memory_available_bytes: system.available_memory(),
            disks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HostInventory;

    #[test]
    fn inventory_has_supported_schema_version() {
        let inventory = HostInventory::collect();
        assert_eq!(inventory.schema_version, 1);
    }

    #[test]
    fn inventory_reports_consistent_memory_values() {
        let inventory = HostInventory::collect();
        assert!(inventory.memory_available_bytes <= inventory.memory_total_bytes);
    }

    #[test]
    fn inventory_serializes_to_json() {
        let inventory = HostInventory::collect();
        let json = serde_json::to_value(inventory).expect("inventory should serialize");
        assert_eq!(json["schema_version"], 1);
        assert!(json["disks"].is_array());
    }
}
