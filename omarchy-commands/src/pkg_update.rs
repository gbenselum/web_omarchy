use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "omarchy-pkg-update", about = "Update packages")]
struct Args {
    #[arg(short, long)]
    packages: Vec<String>,
    
    #[arg(long, default_value = "pacman")]
    manager: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.packages.is_empty() {
        eprintln!("No packages specified");
        std::process::exit(1);
    }
    
    let status = match args.manager.as_str() {
        "pacman" => run_pacman(&args.packages),
        "yay" => run_yay(&args.packages),
        "paru" => run_paru(&args.packages),
        _ => {
            eprintln!("Unknown manager: {}", args.manager);
            std::process::exit(1);
        }
    }?;
    
    std::process::exit(status.code().unwrap_or(1));
}

fn run_pacman(packages: &[String]) -> Result<std::process::ExitStatus> {
    let mut cmd = std::process::Command::new("pacman");
    cmd.args(["-S", "--noconfirm"]);
    cmd.args(packages);
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let status = cmd.status().context("Failed to run pacman")?;
    Ok(status)
}

fn run_yay(packages: &[String]) -> Result<std::process::ExitStatus> {
    let mut cmd = std::process::Command::new("yay");
    cmd.args(["-S", "--noconfirm"]);
    cmd.args(packages);
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let status = cmd.status().context("Failed to run yay")?;
    Ok(status)
}

fn run_paru(packages: &[String]) -> Result<std::process::ExitStatus> {
    let mut cmd = std::process::Command::new("paru");
    cmd.args(["-S", "--noconfirm"]);
    cmd.args(packages);
    cmd.stdin(std::process::Stdio::inherit());
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let status = cmd.status().context("Failed to run paru")?;
    Ok(status)
}