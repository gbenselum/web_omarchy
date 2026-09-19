use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sysinfo::{System};

#[derive(Parser, Debug)]
#[command(name = "omarchy-system-info", about = "Get system information")]
struct Args {
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(Serialize, Deserialize)]
struct SystemInfo {
    hostname: String,
    kernel: String,
    uptime: String,
    packages: u32,
    cpu_usage: f32,
    memory_total: u64,
    memory_used: u64,
    memory_free: u64,
    swap_total: u64,
    swap_used: u64,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    // Wait a bit for CPU usage
    std::thread::sleep(std::time::Duration::from_millis(100));
    sys.refresh_cpu_usage();
    
    let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let kernel = System::kernel_version().unwrap_or_else(|| "unknown".to_string());
    let uptime = format_uptime(System::uptime());
    
    // Count packages (pacman)
    let packages = count_packages().unwrap_or(0);
    
    // CPU usage
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    
    // Memory
    let memory_total = sys.total_memory();
    let memory_used = sys.used_memory();
    let memory_free = sys.free_memory();
    let swap_total = sys.total_swap();
    let swap_used = sys.used_swap();
    
    let info = SystemInfo {
        hostname,
        kernel,
        uptime,
        packages,
        cpu_usage,
        memory_total,
        memory_used,
        memory_free,
        swap_total,
        swap_used,
    };
    
    println!("{}", serde_json::to_string_pretty(&info)?);
    Ok(())
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn count_packages() -> Result<u32> {
    let output = std::process::Command::new("pacman")
        .args(["-Q"])
        .output()
        .context("Failed to run pacman")?;
    
    let count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .count() as u32;
    Ok(count)
}