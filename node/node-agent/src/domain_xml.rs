#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainSpec {
    pub name: String,
    pub vcpu: u16,
    pub memory_mib: u32,
    pub disk_path: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DomainSpecError {
    #[error("domain name must be 1-63 lowercase ASCII letters, digits, or hyphens")]
    InvalidName,
    #[error("vCPU must be between 1 and 64")]
    InvalidVcpu,
    #[error("memory must be between 256 and 262144 MiB")]
    InvalidMemory,
    #[error("disk path must be inside the CloudForge managed disk pool")]
    InvalidDiskPath,
}

impl DomainSpec {
    pub fn validate(&self) -> Result<(), DomainSpecError> {
        if self.name.is_empty()
            || self.name.len() > 63
            || !self.name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || !self.name.as_bytes()[0].is_ascii_lowercase()
        {
            return Err(DomainSpecError::InvalidName);
        }
        if !(1..=64).contains(&self.vcpu) {
            return Err(DomainSpecError::InvalidVcpu);
        }
        if !(256..=262_144).contains(&self.memory_mib) {
            return Err(DomainSpecError::InvalidMemory);
        }
        let prefix = "/var/lib/cloudforge/pools/default/";
        let relative = self
            .disk_path
            .strip_prefix(prefix)
            .ok_or(DomainSpecError::InvalidDiskPath)?;
        if relative.is_empty()
            || relative.contains('/')
            || relative == "."
            || relative == ".."
            || !relative.ends_with(".qcow2")
            || !relative
                .bytes()
                .all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'_' | b'.')
                })
        {
            return Err(DomainSpecError::InvalidDiskPath);
        }
        Ok(())
    }

    pub fn render_libvirt_xml(&self) -> Result<String, DomainSpecError> {
        self.validate()?;
        let memory_kib = u64::from(self.memory_mib) * 1024;
        Ok(format!(
            r#"<domain type='kvm'>
  <name>{}</name>
  <memory unit='KiB'>{}</memory>
  <currentMemory unit='KiB'>{}</currentMemory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='x86_64' machine='q35'>hvm</type>
    <boot dev='hd'/>
  </os>
  <features>
    <acpi/>
    <apic/>
  </features>
  <cpu mode='host-passthrough' check='none'/>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <controller type='scsi' index='0' model='virtio-scsi'/>
    <disk type='file' device='disk'>
      <driver name='qemu' type='qcow2'/>
      <source file='{}'/>
      <target dev='sda' bus='scsi'/>
    </disk>
    <console type='pty'>
      <target type='serial' port='0'/>
    </console>
  </devices>
</domain>
"#,
            xml_escape(&self.name),
            memory_kib,
            memory_kib,
            self.vcpu,
            xml_escape(&self.disk_path),
        ))
    }
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::{DomainSpec, DomainSpecError};

    fn valid_spec() -> DomainSpec {
        DomainSpec {
            name: "cf-test-vm".to_owned(),
            vcpu: 2,
            memory_mib: 2048,
            disk_path: "/var/lib/cloudforge/pools/default/root.qcow2".to_owned(),
        }
    }

    #[test]
    fn renders_valid_q35_kvm_domain_xml() {
        let xml = valid_spec().render_libvirt_xml().unwrap();
        assert!(xml.contains("<domain type='kvm'>"));
        assert!(xml.contains("<type arch='x86_64' machine='q35'>hvm</type>"));
        assert!(xml.contains("<vcpu placement='static'>2</vcpu>"));
        assert!(xml.contains("<memory unit='KiB'>2097152</memory>"));
        assert!(xml.contains("type='qcow2'"));
        assert!(xml.contains("virtio-scsi"));
    }

    #[test]
    fn rejects_invalid_names_and_resource_limits() {
        let mut spec = valid_spec();
        spec.name = "../bad".to_owned();
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidName));
        spec = valid_spec();
        spec.vcpu = 0;
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidVcpu));
        spec = valid_spec();
        spec.memory_mib = 128;
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidMemory));
    }

    #[test]
    fn confines_disk_paths_to_single_managed_pool_filename() {
        let mut spec = valid_spec();
        spec.disk_path = "/etc/passwd".to_owned();
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidDiskPath));
        spec.disk_path = "/var/lib/cloudforge/pools/default/../escape.qcow2".to_owned();
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidDiskPath));
        spec.disk_path = "/var/lib/cloudforge/pools/default/subdir/disk.qcow2".to_owned();
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidDiskPath));
    }

    #[test]
    fn xml_values_are_escaped() {
        let mut spec = valid_spec();
        spec.name = "vm&safe".to_owned();
        assert_eq!(spec.validate(), Err(DomainSpecError::InvalidName));
    }
}
