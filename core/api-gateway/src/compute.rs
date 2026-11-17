use axum::Json;
use serde::Serialize;

/// Lifecycle states exposed by the compute control plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderState {
    Planned,
}

/// Public, truthful description of a compute provider.
/// This is metadata only: it never starts a hypervisor or creates a VM.
#[derive(Debug, Serialize)]
pub struct ComputeProviderInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub state: ProviderState,
    pub virtualization: &'static str,
    pub requires: &'static [&'static str],
    pub supports: &'static [&'static str],
    pub description: &'static str,
}

fn provider_catalog() -> Vec<ComputeProviderInfo> {
    vec![
        ComputeProviderInfo {
            id: "qemu-kvm",
            name: "QEMU/KVM",
            state: ProviderState::Planned,
            virtualization: "hardware_virtualization",
            requires: &[
                "linux_host",
                "qemu_system",
                "kvm_device",
                "trusted_cloudforge_node",
                "scheduler",
                "storage_provider",
                "network_provider",
            ],
            supports: &[
                "virtual_machines",
                "cpu_and_memory_configuration",
                "virtual_disks",
                "virtual_network_interfaces",
            ],
            description: "Target virtual-machine provider. Execution, node orchestration, storage, networking, and scheduling are not implemented yet.",
        },
    ]
}

/// Read-only provider catalog. No host commands are executed.
pub async fn list_providers() -> Json<Vec<ComputeProviderInfo>> {
    Json(provider_catalog())
}

#[cfg(test)]
mod tests {
    use super::{provider_catalog, ProviderState};

    #[test]
    fn qemu_provider_is_not_advertised_as_available() {
        let providers = provider_catalog();
        let qemu = providers
            .iter()
            .find(|provider| provider.id == "qemu-kvm")
            .unwrap();
        assert_eq!(qemu.state, ProviderState::Planned);
    }

    #[test]
    fn qemu_provider_declares_safety_prerequisites() {
        let providers = provider_catalog();
        let qemu = providers
            .iter()
            .find(|provider| provider.id == "qemu-kvm")
            .unwrap();
        for requirement in [
            "linux_host",
            "kvm_device",
            "trusted_cloudforge_node",
            "scheduler",
            "storage_provider",
            "network_provider",
        ] {
            assert!(qemu.requires.contains(&requirement));
        }
    }
}
