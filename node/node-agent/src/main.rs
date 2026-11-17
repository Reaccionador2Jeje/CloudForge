mod compute;
mod identity;
mod inventory;

use std::env;

use identity::IdentityConfig;
use inventory::HostInventory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::args().any(|argument| argument == "--compute-preflight") {
        let report = compute::collect_preflight();
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !report.provider_ready {
            std::process::exit(2);
        }
        return Ok(());
    }

    if env::args().any(|argument| argument == "--check-identity") {
        let config = IdentityConfig::from_env()?;
        println!("{}", serde_json::to_string_pretty(&config.safe_summary())?);
        return Ok(());
    }

    let inventory = HostInventory::collect();
    println!("{}", serde_json::to_string_pretty(&inventory)?);
    Ok(())
}
