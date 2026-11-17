use serde::Serialize;
use sysinfo::Disks;

const GIB: u64 = 1024 * 1024 * 1024;
const DEFAULT_POOL: &str = "/var/lib/cloudforge/pools/default";
const DEFAULT_RESERVE_GIB: u64 = 10;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StorageCapacityReport {
    pub schema_version: u8,
    pub pool_path: String,
    pub matched_mount_point: Option<String>,
    pub total_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub requested_virtual_bytes: u64,
    pub reserve_bytes: u64,
    pub required_available_bytes: u64,
    pub capacity_sufficient: bool,
    pub blockers: Vec<&'static str>,
}

/// Selects the most specific mounted filesystem containing the pool path.
pub fn collect_capacity(requested_gib: u64) -> StorageCapacityReport {
    let disks = Disks::new_with_refreshed_list();
    let pool = std::path::Path::new(DEFAULT_POOL);
    let matching_disk = disks
        .iter()
        .filter(|disk| pool.starts_with(disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count());

    let requested_virtual_bytes = requested_gib.saturating_mul(GIB);
    let reserve_bytes = DEFAULT_RESERVE_GIB * GIB;
    let required_available_bytes = requested_virtual_bytes.saturating_add(reserve_bytes);

    let matched_mount_point = matching_disk.map(|disk| disk.mount_point().display().to_string());
    let total_bytes = matching_disk.map(|disk| disk.total_space());
    let available_bytes = matching_disk.map(|disk| disk.available_space());

    let mut blockers = Vec::new();
    if requested_gib == 0 || requested_gib > 2048 {
        blockers.push("requested_size_out_of_range");
    }
    if matching_disk.is_none() {
        blockers.push("pool_filesystem_not_found");
    }
    if let Some(available) = available_bytes {
        if available < required_available_bytes {
            blockers.push("insufficient_physical_capacity");
        }
    }

    let capacity_sufficient = blockers.is_empty();

    StorageCapacityReport {
        schema_version: 1,
        pool_path: DEFAULT_POOL.to_owned(),
        matched_mount_point,
        total_bytes,
        available_bytes,
        requested_virtual_bytes,
        reserve_bytes,
        required_available_bytes,
        capacity_sufficient,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::{StorageCapacityReport, GIB};

    #[test]
    fn capacity_report_requires_requested_size_plus_reserve() {
        let report = StorageCapacityReport {
            schema_version: 1,
            pool_path: "/var/lib/cloudforge/pools/default".to_owned(),
            matched_mount_point: Some("/".to_owned()),
            total_bytes: Some(100 * GIB),
            available_bytes: Some(50 * GIB),
            requested_virtual_bytes: 20 * GIB,
            reserve_bytes: 10 * GIB,
            required_available_bytes: 30 * GIB,
            capacity_sufficient: true,
            blockers: Vec::new(),
        };
        assert!(report.capacity_sufficient);
        assert_eq!(report.required_available_bytes, 30 * GIB);
    }

    #[test]
    fn capacity_report_schema_can_represent_insufficient_space() {
        let report = StorageCapacityReport {
            schema_version: 1,
            pool_path: "/var/lib/cloudforge/pools/default".to_owned(),
            matched_mount_point: Some("/".to_owned()),
            total_bytes: Some(100 * GIB),
            available_bytes: Some(5 * GIB),
            requested_virtual_bytes: 20 * GIB,
            reserve_bytes: 10 * GIB,
            required_available_bytes: 30 * GIB,
            capacity_sufficient: false,
            blockers: vec!["insufficient_physical_capacity"],
        };
        assert!(!report.capacity_sufficient);
        assert_eq!(report.blockers[0], "insufficient_physical_capacity");
    }
}
