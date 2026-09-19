use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use sysinfo::Networks;

#[derive(Parser, Debug)]
#[command(name = "omarchy-network-status", about = "Get network interface status")]
struct Args {
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(Serialize, Deserialize)]
struct NetworkInterface {
    name: String,
    status: String,
    ip: String,
    mac: String,
    rx_bytes: u64,
    tx_bytes: u64,
    rx_packets: u64,
    tx_packets: u64,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    
    let mut networks = Networks::new_with_refreshed_list();
    // Wait a bit and refresh to get actual traffic
    std::thread::sleep(std::time::Duration::from_millis(100));
    networks.refresh();
    
    let mut interfaces = Vec::new();
    
    for (name, data) in &networks {
        let status = if data.received() > 0 || data.transmitted() > 0 { "up" } else { "down" };
        
        // Get IP and MAC from system files
        let (ip, mac) = get_interface_details(name);
        
        interfaces.push(NetworkInterface {
            name: name.to_string(),
            status: status.to_string(),
            ip,
            mac,
            rx_bytes: data.received(),
            tx_bytes: data.transmitted(),
            rx_packets: data.packets_received(),
            tx_packets: data.packets_transmitted(),
        });
    }
    
    println!("{}", serde_json::to_string_pretty(&interfaces)?);
    Ok(())
}

fn get_interface_details(name: &str) -> (String, String) {
    // Try to get IP from ip addr show
    let ip = std::process::Command::new("ip")
        .args(["-4", "addr", "show", name])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .find(|l| l.contains("inet "))
                .and_then(|l| l.split_whitespace().nth(1))
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "N/A".to_string());
    
    // Try to get MAC from /sys/class/net
    let mac = fs::read_to_string(format!("/sys/class/net/{}/address", name))
        .ok()
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "N/A".to_string());
    
    (ip, mac)
}