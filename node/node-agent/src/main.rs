mod inventory;

use inventory::HostInventory;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let inventory = HostInventory::collect();
    println!("{}", serde_json::to_string_pretty(&inventory)?);
    Ok(())
}
