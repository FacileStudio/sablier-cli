mod api;
mod config;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sablier", about = "Terminal client for Sablier time tracker")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Authenticate with your Sablier instance")]
    Login,
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
        Command::Login => cmd_login().await,
        Command::Status => cmd_status().await,
        Command::Stop => cmd_stop().await,
        Command::Pause => cmd_pause().await,
        Command::Resume => cmd_resume().await,
        Command::Projects => cmd_projects().await,
    }
}

async fn cmd_login() -> Result<()> {
    let mut cfg = config::Config::load_or_default();

    if cfg.server_url.is_empty() {
        eprint!("Server URL: ");
        let mut url = String::new();
        std::io::stdin().read_line(&mut url)?;
        cfg.server_url = url.trim().to_string();
    }

    eprint!("Email: ");
    let mut email = String::new();
    std::io::stdin().read_line(&mut email)?;
    let email = email.trim().to_string();

    let password = rpassword::prompt_password("Password: ")?;

    let client = api::ApiClient::new(&cfg.server_url, "");
    let resp = client.login(&email, &password).await?;

    cfg.token = resp.token;
    cfg.save()?;

    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    let user = client.me().await?;

    let name = if user.name.is_empty() {
        &user.email
    } else {
        &user.name
    };
    println!("Logged in as {}", name);
    Ok(())
}

async fn cmd_status() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);

    match client.running_entry().await? {
        Some(entry) => {
            let elapsed = entry.elapsed_display();
            let status = entry.status_label();
            println!("{} {}", elapsed, status);
        }
        None => println!("No timer running."),
    }
    Ok(())
}

async fn cmd_stop() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    let entry = client.stop().await?;
    println!("Stopped. Total: {}", entry.elapsed_display());
    Ok(())
}

async fn cmd_pause() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    client.pause().await?;
    println!("Timer paused.");
    Ok(())
}

async fn cmd_resume() -> Result<()> {
    let cfg = config::Config::load()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    client.resume().await?;
    println!("Timer resumed.");
    Ok(())
}

async fn cmd_projects() -> Result<()> {
    let cfg = config::Config::load()?;
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
