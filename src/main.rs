mod api;
mod config;
mod tui;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sablier", about = "Terminal client for Sablier time tracker\n\nGenerate your API token at your Sablier dashboard (Profile > API Token),\nthen add it to ~/.sablier.yml")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Show the currently running timer")]
    Status,
    #[command(about = "Stop the running timer")]
    Stop,
    #[command(about = "Pause the running timer")]
    Pause,
    #[command(about = "Resume the paused timer")]
    Resume,
    #[command(about = "List projects")]
    Projects,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => tui::run().await,
        Some(cmd) => run_command(cmd).await,
    }
}

async fn run_command(cmd: Command) -> Result<()> {
    match cmd {
        Command::Status => cmd_status().await,
        Command::Stop => cmd_stop().await,
        Command::Pause => cmd_pause().await,
        Command::Resume => cmd_resume().await,
        Command::Projects => cmd_projects().await,
    }
}

fn load_authed_config() -> Result<config::Config> {
    let cfg = config::Config::load()?;
    if cfg.token.is_empty() {
        bail!(
            "No API token configured.\n\
             Generate one at your Sablier dashboard (Profile > API Token),\n\
             then add it to ~/.sablier.yml:\n\n  \
             server_url: https://your-instance.example.com\n  \
             token: your-token-here"
        );
    }
    Ok(cfg)
}

async fn cmd_status() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);

    match client.running_entry().await? {
        Some(entry) => {
            let elapsed = entry.elapsed_display();
            let status = entry.status_label();
            let projects = client.projects().await.unwrap_or_default();
            let project_name = projects
                .iter()
                .find(|p| p.id == entry.project_id)
                .map(|p| p.name.as_str())
                .unwrap_or("?");
            println!("{} {} — {}", elapsed, status, project_name);
        }
        None => println!("No timer running."),
    }
    Ok(())
}

async fn cmd_stop() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    let entry = client.stop().await?;
    println!("Stopped. Total: {}", entry.elapsed_display());
    Ok(())
}

async fn cmd_pause() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    client.pause().await?;
    println!("Timer paused.");
    Ok(())
}

async fn cmd_resume() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    client.resume().await?;
    println!("Timer resumed.");
    Ok(())
}

async fn cmd_projects() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    let projects = client.projects().await?;

    if projects.is_empty() {
        println!("No projects.");
        return Ok(());
    }

    for p in &projects {
        if p.description.is_empty() {
            println!("  {}", p.name);
        } else {
            println!("  {}  — {}", p.name, p.description);
        }
    }
    Ok(())
}
