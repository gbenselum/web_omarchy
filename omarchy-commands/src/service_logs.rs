use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "omarchy-service-logs", about = "Get systemd service logs")]
struct Args {
    #[arg(short, long)]
    unit: String,
    
    #[arg(short, long, default_value = "100")]
    lines: u32,
    
    #[arg(short, long)]
    user: bool,
    
    #[arg(long)]
    follow: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    let mut cmd = std::process::Command::new("journalctl");
    
    if args.user {
        cmd.arg("--user");
    }
    
    cmd.args(["-u", &args.unit, "-n", &args.lines.to_string(), "--no-pager"]);
    
    if args.follow {
        cmd.arg("-f");
    }
    
    let status = cmd.status().context("Failed to run journalctl")?;
    std::process::exit(status.code().unwrap_or(1));
}