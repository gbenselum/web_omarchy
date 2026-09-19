use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use sysinfo::{Disks};

#[derive(Parser, Debug)]
#[command(name = "omarchy-storage-list", about = "List storage devices")]
struct Args {
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(Serialize, Deserialize)]
struct StorageDevice {
    name: String,
    size: String,
    fstype: String,
    mountpoint: String,
    usage_percent: f32,
    available: String,
    smart_status: String,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    
    let disks = Disks::new_with_refreshed_list();
    let mut devices = Vec::new();
    
    for disk in disks.list() {
        let name = disk.name().to_string_lossy().to_string();
        let total = disk.total_space();
        let available = disk.available_space();
        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 };
        
        let fstype = disk.file_system().to_string_lossy().to_string();
        let mountpoint = disk.mount_point().to_string_lossy().to_string();
        
        // Get device name (e.g., sda, nvme0n1)
        let dev_name = get_device_name(&mountpoint).unwrap_or(name.clone());
        
        // Check SMART status
        let smart_status = check_smart(&dev_name);
        
        devices.push(StorageDevice {
            name: dev_name,
            size: format_bytes(total),
            fstype,
            mountpoint: if mountpoint.is_empty() { "-".to_string() } else { mountpoint },
            usage_percent,
            available: format_bytes(available),
            smart_status,
        });
    }
    
    println!("{}", serde_json::to_string_pretty(&devices)?);
    Ok(())
}

fn get_device_name(mountpoint: &str) -> Option<String> {
    if mountpoint.is_empty() || mountpoint == "-" {
        return None;
    }
    let output = std::process::Command::new("findmnt")
        .args(["-n", "-o", "SOURCE", mountpoint])
        .output()
        .ok()?;
    String::from_utf8(output.stdout).ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn check_smart(device: &str) -> String {
    let output = std::process::Command::new("smartctl")
        .args(["-H", &format!("/dev/{}", device)])
        .output();
    
    match output {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            if s.contains("PASSED") { "OK" } else { "WARN" }.to_string()
        }
        _ => "N/A".to_string(),
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}