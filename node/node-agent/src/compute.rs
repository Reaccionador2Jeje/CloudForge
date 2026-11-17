use serde::Serialize;
use std::process::{Command, Output};

#[derive(Debug, Clone, Serialize)]
pub struct LibvirtDomain {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComputePreflight {
    pub schema_version: u32,
    pub supported_host: bool,
    pub kvm_device_available: bool,
    pub qemu_binary_available: bool,
    pub libvirt_cli_available: bool,
    pub libvirt_system_connection: bool,
    pub provider_ready: bool,
    pub domains: Vec<LibvirtDomain>,
    pub blockers: Vec<&'static str>,
}

#[cfg(target_os = "linux")]
fn is_kvm_character_device() -> bool {
    use std::os::unix::fs::FileTypeExt;
    std::fs::metadata("/dev/kvm")
        .map(|metadata| metadata.file_type().is_char_device())
        .unwrap_or(false)
}

#[cfg(not(target_os = "linux"))]
fn is_kvm_character_device() -> bool {
    false
}

fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn command_output(program: &str, args: &[&str]) -> Option<Output> {
    Command::new(program).args(args).output().ok()
}

fn parse_domains(output: &str) -> Vec<LibvirtDomain> {
    output
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| LibvirtDomain {
            name: name.to_owned(),
        })
        .collect()
}

/// Performs a read-only preflight against the local system libvirt connection.
/// No VM lifecycle or host configuration commands are executed.
pub fn collect_preflight() -> ComputePreflight {
    let supported_host = cfg!(target_os = "linux");
    let kvm_device_available = supported_host && is_kvm_character_device();
    let qemu_binary_available =
        supported_host && command_succeeds("qemu-system-x86_64", &["--version"]);
    let libvirt_cli_available = supported_host && command_succeeds("virsh", &["--version"]);
    let libvirt_system_connection =
        libvirt_cli_available && command_succeeds("virsh", &["-c", "qemu:///system", "version"]);

    let domain_output = if libvirt_system_connection {
        command_output(
            "virsh",
            &["-c", "qemu:///system", "list", "--all", "--name"],
        )
    } else {
        None
    };

    let domain_inventory_available = domain_output.as_ref().is_some_and(|output| {
        output.status.success() && String::from_utf8(output.stdout.clone()).is_ok()
    });
    let domains = domain_output
        .as_ref()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout.clone()).ok())
        .map(|output| parse_domains(&output))
        .unwrap_or_default();

    let mut blockers = Vec::new();
    if !supported_host {
        blockers.push("linux_host_required");
    }
    if !kvm_device_available {
        blockers.push("kvm_device_unavailable");
    }
    if !qemu_binary_available {
        blockers.push("qemu_system_x86_64_unavailable");
    }
    if !libvirt_cli_available {
        blockers.push("libvirt_virsh_unavailable");
    }
    if !libvirt_system_connection {
        blockers.push("libvirt_system_connection_unavailable");
    }
    if libvirt_system_connection && !domain_inventory_available {
        blockers.push("libvirt_domain_inventory_failed");
    }

    let provider_ready = blockers.is_empty();

    ComputePreflight {
        schema_version: 1,
        supported_host,
        kvm_device_available,
        qemu_binary_available,
        libvirt_cli_available,
        libvirt_system_connection,
        provider_ready,
        domains,
        blockers,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_domains;

    #[test]
    fn parses_libvirt_domain_names_without_empty_rows() {
        let domains = parse_domains("ubuntu-dev\n\nwindows-lab\n");
        assert_eq!(domains.len(), 2);
        assert_eq!(domains[0].name, "ubuntu-dev");
        assert_eq!(domains[1].name, "windows-lab");
    }

    #[test]
    fn empty_libvirt_output_has_no_domains() {
        assert!(parse_domains("\n  \n").is_empty());
    }
}
