mod compute;
pub mod domain_xml;
mod identity;
mod inventory;

use std::env;

use identity::IdentityConfig;
use inventory::HostInventory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = env::args().collect();
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
