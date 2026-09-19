use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "omarchy-service-control", about = "Control systemd services")]
struct Args {
    #[arg(short, long)]
    action: String,
    
    #[arg(short, long)]
    unit: String,
    
    #[arg(short, long)]
    user: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let valid_actions = ["start", "stop", "restart", "enable", "disable", "status"];
    if !valid_actions.contains(&args.action.as_str()) {
        eprintln!("Invalid action: {}. Valid: {}", args.action, valid_actions.join(", "));
        std::process::exit(1);
    }
    
    let mut cmd = std::process::Command::new("systemctl");
    
    if args.user {
        cmd.arg("--user");
    }
    
    cmd.args([&args.action, &args.unit]);
    
    let status = cmd.status().context("Failed to run systemctl")?;
    std::process::exit(status.code().unwrap_or(1));
}