use serde::Serialize;
use std::path::Path;
use std::process::Command;
use sysinfo::System;

/// Read-only description of the environment in which the agent is running.
/// This is diagnostic information, not proof that the machine can host workloads.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeProfile {
    pub schema_version: u32,
    pub operating_system: String,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub architecture: String,
    pub execution_environment: &'static str,
    pub proot_detected: bool,
    pub kvm_device_available: bool,
    pub qemu_binary_available: bool,
    pub libvirt_cli_available: bool,
    pub docker_cli_available: bool,
    pub podman_cli_available: bool,
    pub systemd_runtime_active: bool,
    pub capabilities: Vec<&'static str>,
    pub limitations: Vec<&'static str>,
}

fn command_available(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn detect_proot() -> bool {
    let environment_marker = std::env::vars().any(|(key, _)| {
        let key = key.to_ascii_uppercase();
        key.starts_with("PROOT_") || key == "PROOT"
    });

    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo")
        .unwrap_or_default()
        .to_ascii_lowercase();

    let termux_mount_marker = mountinfo.contains("/data/data/com.termux/")
        || mountinfo.contains("/data/user/0/com.termux/");

    environment_marker || termux_mount_marker
}

#[cfg(target_os = "linux")]
fn detect_kvm() -> bool {
    use std::os::unix::fs::FileTypeExt;

    std::fs::metadata("/dev/kvm")
        .is_ok_and(|metadata| metadata.file_type().is_char_device())
}

#[cfg(not(target_os = "linux"))]
fn detect_kvm() -> bool {
    false
}

fn matching_qemu_binary(architecture: &str) -> Option<&'static str> {
    match architecture {
        "aarch64" => Some("qemu-system-aarch64"),
        "x86_64" => Some("qemu-system-x86_64"),
        _ => None,
    }
}

pub fn collect_profile() -> RuntimeProfile {
    let operating_system = std::env::consts::OS.to_owned();
    let architecture = std::env::consts::ARCH.to_owned();
    let proot_detected = detect_proot();

    let execution_environment = if proot_detected {
        "proot"
    } else if operating_system == "linux" {
        "linux"
    } else {
        "other"
    };

    let kvm_device_available = detect_kvm();
    let qemu_binary_available = matching_qemu_binary(&architecture)
        .is_some_and(command_available);
    let libvirt_cli_available = command_available("virsh");
    let docker_cli_available = command_available("docker");
    let podman_cli_available = command_available("podman");
    let systemd_runtime_active = Path::new("/run/systemd/system").is_dir();

    let mut capabilities = vec!["read_only_inventory"];

    if operating_system == "linux" {
        capabilities.push("linux_userspace");
    }
    if matches!(architecture.as_str(), "aarch64" | "x86_64") {
        capabilities.push("supported_cpu_architecture");
    }
    if qemu_binary_available {
        capabilities.push("qemu_binary");
    }
    if kvm_device_available {
        capabilities.push("kvm_device");
    }
    if libvirt_cli_available {
        capabilities.push("libvirt_cli");
    }
    if docker_cli_available {
        capabilities.push("docker_cli");
    }
    if podman_cli_available {
        capabilities.push("podman_cli");
    }
    if systemd_runtime_active {
        capabilities.push("systemd_runtime");
    }

    let mut limitations = Vec::new();

    if proot_detected {
        limitations.push("proot_is_not_a_full_virtual_machine_host");
        limitations.push("host_kernel_is_shared_with_android");
    }
    if !kvm_device_available {
        limitations.push("kvm_device_unavailable");
    }
    if !qemu_binary_available {
        limitations.push("architecture_matching_qemu_binary_unavailable");
    }
    if !libvirt_cli_available {
        limitations.push("libvirt_cli_unavailable");
    }
    if !systemd_runtime_active {
        limitations.push("systemd_runtime_not_detected");
    }
    if !docker_cli_available && !podman_cli_available {
        limitations.push("container_runtime_cli_unavailable");
    }

    RuntimeProfile {
        schema_version: 1,
        operating_system,
        os_name: System::name(),
        os_version: System::os_version(),
        kernel_version: System::kernel_version(),
        architecture,
        execution_environment,
        proot_detected,
        kvm_device_available,
        qemu_binary_available,
        libvirt_cli_available,
        docker_cli_available,
        podman_cli_available,
        systemd_runtime_active,
        capabilities,
        limitations,
    }
}

#[cfg(test)]
mod tests {
    use super::matching_qemu_binary;

    #[test]
    fn selects_qemu_binary_for_supported_architectures() {
        assert_eq!(matching_qemu_binary("aarch64"), Some("qemu-system-aarch64"));
        assert_eq!(matching_qemu_binary("x86_64"), Some("qemu-system-x86_64"));
        assert_eq!(matching_qemu_binary("arm"), None);
    }
}
