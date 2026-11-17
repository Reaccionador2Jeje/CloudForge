mod compute;
pub mod domain_xml;
mod identity;
mod inventory;
pub mod provisioning;
pub mod storage;
pub mod storage_capacity;
pub mod storage_allocation;
pub mod storage_preflight;

use std::env;

use identity::IdentityConfig;
use inventory::HostInventory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--plan-storage-allocation")
    {
        if arguments.len() != 8 {
            eprintln!("usage: cloudforge-node-agent --plan-storage-allocation <project-uuid> <quota-gib> <used-gib> <reserved-gib> <requested-gib> <node-available-gib>");
            std::process::exit(64);
        }
        let request = storage_allocation::StorageAllocationRequest {
            project_id: arguments[2].clone(),
            quota_gib: arguments[3].parse()?,
            used_gib: arguments[4].parse()?,
            reserved_gib: arguments[5].parse()?,
            requested_gib: arguments[6].parse()?,
            node_available_gib: arguments[7].parse()?,
        };
        let plan = request.build_plan()?;
        println!("{}", serde_json::to_string_pretty(&plan)?);
        if !plan.blockers.is_empty() {
            std::process::exit(2);
        }
        return Ok(());
    }

    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--storage-capacity")
    {
        if arguments.len() != 3 {
            eprintln!("usage: cloudforge-node-agent --storage-capacity <requested-size-gib>");
            std::process::exit(64);
        }
        let requested_gib: u64 = arguments[2].parse()?;
        let report = storage_capacity::collect_capacity(requested_gib);
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report.capacity_sufficient {
            std::process::exit(2);
        }
        return Ok(());
    }

    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--storage-preflight")
    {
        if arguments.len() != 2 {
            eprintln!("usage: cloudforge-node-agent --storage-preflight");
            std::process::exit(64);
        }
        let report = storage_preflight::collect_preflight();
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report.provider_ready {
            std::process::exit(2);
        }
        return Ok(());
    }

    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--plan-vm")
    {
        if arguments.len() != 7 {
            eprintln!(
                "usage: cloudforge-node-agent --plan-vm <name> <vcpu> <memory-mib> <disk-name> <disk-size-gib>"
            );
            std::process::exit(64);
        }
        let request = provisioning::VmProvisioningRequest {
            name: arguments[2].clone(),
            vcpu: arguments[3].parse()?,
            memory_mib: arguments[4].parse()?,
            disk_name: arguments[5].clone(),
            disk_size_gib: arguments[6].parse()?,
        };
        let plan = request.build_plan()?;
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--render-domain-xml")
    {
        if arguments.len() != 6 {
            eprintln!("usage: cloudforge-node-agent --render-domain-xml <name> <vcpu> <memory-mib> <disk-path>");
            std::process::exit(64);
        }
        let spec = domain_xml::DomainSpec {
            name: arguments[2].clone(),
            vcpu: arguments[3].parse()?,
            memory_mib: arguments[4].parse()?,
            disk_path: arguments[5].clone(),
        };
        print!("{}", spec.render_libvirt_xml()?);
        return Ok(());
    }

    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--plan-disk")
    {
        if arguments.len() != 4 {
            eprintln!("usage: cloudforge-node-agent --plan-disk <name> <size-gib>");
            std::process::exit(64);
        }
        let request = storage::DiskRequest {
            name: arguments[2].clone(),
            size_gib: arguments[3].parse()?,
        };
        let plan = request.build_plan()?;
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }

    if arguments
        .iter()
        .any(|argument| argument == "--compute-preflight")
    {
        let report = compute::collect_preflight();
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report.provider_ready {
            std::process::exit(2);
        }
        return Ok(());
    }

    if arguments
        .iter()
        .any(|argument| argument == "--check-identity")
    {
        let config = IdentityConfig::from_env()?;
        println!("{}", serde_json::to_string_pretty(&config.safe_summary())?);
        return Ok(());
    }

    let inventory = HostInventory::collect();
    println!("{}", serde_json::to_string_pretty(&inventory)?);
    Ok(())
}
