use axum::Json;
use serde::Serialize;

/// Read-only guest installation profile. This is planning metadata only.
#[derive(Debug, Serialize)]
pub struct GuestInstallProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub os_family: &'static str,
    pub os_version: &'static str,
    pub architecture: &'static str,
    pub readiness: &'static str,
    pub execution_enabled: bool,
    pub firmware: &'static str,
    pub machine_type: &'static str,
    pub installer_artifact: &'static str,
    pub disk_bus: &'static str,
    pub network_model: &'static str,
    pub driver_stage: &'static str,
    pub secure_boot: &'static str,
    pub tpm: &'static str,
    pub license_required: bool,
    pub installation_stages: &'static [&'static str],
    pub limitations: &'static [&'static str],
}

fn windows_profiles() -> Vec<GuestInstallProfile> {
    let limitations = &[
        "QEMU/KVM execution is not implemented",
        "Firmware, machine type, and virtual devices require integration testing",
        "VirtIO driver delivery and installation are not implemented",
        "No Microsoft installation media is hosted or redistributed",
        "A valid Windows Server license and activation path are required",
    ];

    [
        ("windows-server-2025", "Windows Server 2025", "2025"),
        ("windows-server-2022", "Windows Server 2022", "2022"),
    ]
    .into_iter()
    .map(|(id, display_name, os_version)| GuestInstallProfile {
        id,
        display_name,
        os_family: "windows-server",
        os_version,
        architecture: "x86_64",
        readiness: "planned",
        execution_enabled: false,
        firmware: "uefi",
        machine_type: "q35",
        installer_artifact: "installer_iso",
        disk_bus: "virtio_scsi",
        network_model: "virtio_net",
        driver_stage: "required",
        secure_boot: "requires_validation",
        tpm: "requires_validation",
        license_required: true,
        installation_stages: &[
            "validate_authorized_installer_media",
            "allocate_vm_resources",
            "prepare_virtual_hardware",
            "attach_installer_iso",
            "boot_windows_setup",
            "load_storage_and_network_drivers",
            "complete_guest_installation",
            "verify_guest_health_and_license",
            "convert_to_verified_reusable_image",
        ],
        limitations,
    })
    .collect()
}

/// Returns proposed Windows guest installation settings without starting a VM.
pub async fn list_guest_profiles() -> Json<Vec<GuestInstallProfile>> {
    Json(windows_profiles())
}

#[cfg(test)]
mod tests {
    use super::windows_profiles;

    #[test]
    fn windows_server_profiles_are_planned_and_not_executable() {
        let profiles = windows_profiles();
        assert_eq!(profiles.len(), 2);
        for profile in profiles {
            assert_eq!(profile.os_family, "windows-server");
            assert_eq!(profile.architecture, "x86_64");
            assert_eq!(profile.readiness, "planned");
            assert!(!profile.execution_enabled);
            assert!(profile.license_required);
            assert_eq!(profile.installer_artifact, "installer_iso");
        }
    }

    #[test]
    fn windows_profiles_include_driver_and_verification_stages() {
        let profiles = windows_profiles();
        for profile in profiles {
            assert!(profile
                .installation_stages
                .contains(&"load_storage_and_network_drivers"));
            assert!(profile
                .installation_stages
                .contains(&"verify_guest_health_and_license"));
        }
    }
}
