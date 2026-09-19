use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "omarchy-pkg-check", about = "Check for package updates")]
struct Args {
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(Serialize, Deserialize)]
struct Update {
    name: String,
    current: String,
    new: String,
    size: String,
    repo: String,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    
    // Try checkupdates first (safe, doesn't require root)
    let updates = if let Ok(u) = run_checkupdates() {
        u
    } else if let Ok(u) = run_pacman_qu() {
        u
    } else {
        Vec::new()
    };
    
    println!("{}", serde_json::to_string_pretty(&updates)?);
    Ok(())
}

fn run_checkupdates() -> Result<Vec<Update>> {
    let output = std::process::Command::new("checkupdates")
        .output()
        .context("Failed to run checkupdates")?;
    
    if !output.status.success() && !output.stdout.is_empty() {
        anyhow::bail!("checkupdates failed");
    }
    
    parse_updates(&String::from_utf8_lossy(&output.stdout))
}

fn run_pacman_qu() -> Result<Vec<Update>> {
    let output = std::process::Command::new("pacman")
        .args(["-Qu"])
        .output()
        .context("Failed to run pacman -Qu")?;
    
    parse_updates(&String::from_utf8_lossy(&output.stdout))
}

fn parse_updates(text: &str) -> Result<Vec<Update>> {
    let mut updates = Vec::new();
    
    for line in text.lines() {
        // Format: package current -> new
        // or: package current-version -> new-version
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[2] == "->" {
            let name = parts[0].to_string();
            let current = parts[1].to_string();
            let new = parts[3].to_string();
            
            // Try to get size and repo
            let (size, repo) = get_package_info(&name);
            
            updates.push(Update {
                name,
                current,
                new,
                size,
                repo,
            });
        }
    }
    
    Ok(updates)
}

fn get_package_info(name: &str) -> (String, String) {
    // Try pacman -Si
    let output = std::process::Command::new("pacman")
        .args(["-Si", name])
        .output();
    
    if let Ok(o) = output {
        let text = String::from_utf8_lossy(&o.stdout);
        let mut size = "N/A".to_string();
        let mut repo = "N/A".to_string();
        
        for line in text.lines() {
            if line.starts_with("Download Size") {
                size = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Repository") {
                repo = line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }
        
        return (size, repo);
    }
    
    ("N/A".to_string(), "N/A".to_string())
}