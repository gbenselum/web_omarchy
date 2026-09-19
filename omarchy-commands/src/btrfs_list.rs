use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "omarchy-btrfs-list", about = "List Btrfs snapshots")]
struct Args {
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(Serialize, Deserialize)]
struct Snapshot {
    id: String,
    name: String,
    created: String,
    size: String,
    snapshot_type: String,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    
    // Try snapper first
    let snapshots = if let Ok(snaps) = list_snapper() {
        snaps
    } else if let Ok(snaps) = list_btrfs_subvolumes() {
        snaps
    } else {
        Vec::new()
    };
    
    println!("{}", serde_json::to_string_pretty(&snapshots)?);
    Ok(())
}

fn list_snapper() -> Result<Vec<Snapshot>> {
    let output = std::process::Command::new("snapper")
        .args(["list", "--disable-used-space"])
        .output()
        .context("Failed to run snapper")?;
    
    if !output.status.success() {
        anyhow::bail!("snapper failed");
    }
    
    let text = String::from_utf8_lossy(&output.stdout);
    let mut snapshots = Vec::new();
    
    for line in text.lines().skip(2) { // Skip header
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 6 {
            let id = parts[0].trim().to_string();
            let snapshot_type = parts[1].trim().to_string();
            let created = parts[3].trim().to_string();
            let name = parts[4].trim().to_string();
            
            if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
                snapshots.push(Snapshot {
                    id,
                    name,
                    created,
                    size: "N/A".to_string(),
                    snapshot_type,
                });
            }
        }
    }
    
    Ok(snapshots)
}

fn list_btrfs_subvolumes() -> Result<Vec<Snapshot>> {
    let output = std::process::Command::new("btrfs")
        .args(["subvolume", "list", "/"])
        .output()
        .context("Failed to run btrfs")?;
    
    if !output.status.success() {
        anyhow::bail!("btrfs failed");
    }
    
    let text = String::from_utf8_lossy(&output.stdout);
    let mut snapshots = Vec::new();
    
    for line in text.lines() {
        // Format: ID 257 gen 0 top level 5 path @snapshots/snap-2024-01-01
        if let Some(path_start) = line.find("path ") {
            let path = &line[path_start + 5..];
            let parts: Vec<&str> = path.split('/').collect();
            if let Some(name) = parts.last() {
                // Try to extract date from name
                let id = line.split_whitespace().nth(1).unwrap_or("0").to_string();
                snapshots.push(Snapshot {
                    id,
                    name: name.to_string(),
                    created: "Unknown".to_string(),
                    size: "N/A".to_string(),
                    snapshot_type: "subvolume".to_string(),
                });
            }
        }
    }
    
    Ok(snapshots)
}