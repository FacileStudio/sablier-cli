mod api;
mod config;
mod login;
mod tui;
mod ui;

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "sablier",
    version,
    about = "Terminal client for Sablier time tracking",
    long_about = "Terminal client for Sablier time tracking.\n\nRun with no arguments for the full-screen TUI, or with a subcommand to drive\ntimers from a shell.\n\nGenerate your API token at your Sablier dashboard (Profile > API Token), then\nrun `sablier login` to sign in."
)]
struct Cli {
    #[arg(long, global = true, help = "Disable colored output")]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Sign in through the browser and store the token")]
    Login {
        #[arg(long, help = "Server URL, e.g. https://sablier.facile.studio")]
        server: Option<String>,
    },
    #[command(about = "Start a new timer (interactive project/task picker)")]
    Start {
        #[arg(long, help = "Project ID (skip interactive picker)")]
        project_id: Option<i64>,
        #[arg(long, help = "Task ID (requires --project-id)")]
        task_id: Option<i64>,
    },
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
    #[command(about = "Upgrade sablier to the latest version")]
    Upgrade,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if cli.no_color {
        ui::disable_color();
    }

    let result = match cli.command {
        None => tui::run().await,
        Some(cmd) => run_command(cmd).await,
    };

    if let Err(e) = result {
        ui::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}

async fn run_command(cmd: Command) -> Result<()> {
    match cmd {
        Command::Login { server } => login::run(server).await,
        Command::Start {
            project_id,
            task_id,
        } => cmd_start(project_id, task_id).await,
        Command::Status => cmd_status().await,
        Command::Stop => cmd_stop().await,
        Command::Pause => cmd_pause().await,
        Command::Resume => cmd_resume().await,
        Command::Projects => cmd_projects().await,
        Command::Upgrade => cmd_upgrade().await,
    }
}

fn load_authed_config() -> Result<config::Config> {
    let cfg = config::Config::load()?;
    if cfg.token.is_empty() {
        bail!(
            "no API token configured — generate one at your Sablier dashboard\n  \
             (Profile > API Token), then add it to ~/.sablier.yml:\n\n  \
             server_url: https://your-instance.example.com\n  \
             token: your-token-here"
        );
    }
    Ok(cfg)
}

async fn cmd_start(project_id: Option<i64>, task_id: Option<i64>) -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);

    let (pid, tid) = match (project_id, task_id) {
        (Some(p), Some(t)) => (p, t),
        (Some(p), None) => {
            let tasks = client.tasks(p).await?;
            if tasks.is_empty() {
                bail!("no tasks in project {}", p);
            }
            let names: Vec<String> = tasks.iter().map(|t| t.name.clone()).collect();
            let selection = dialoguer::FuzzySelect::new()
                .with_prompt("Select task")
                .items(&names)
                .default(0)
                .interact()?;
            (p, tasks[selection].id)
        }
        _ => {
            let projects = client.projects().await?;
            if projects.is_empty() {
                bail!("no projects available");
            }
            let proj_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
            let proj_sel = dialoguer::FuzzySelect::new()
                .with_prompt("Select project")
                .items(&proj_names)
                .default(0)
                .interact()?;
            let project = &projects[proj_sel];

            let tasks = client.tasks(project.id).await?;
            if tasks.is_empty() {
                bail!("no tasks in project \"{}\"", project.name);
            }
            let task_names: Vec<String> = tasks.iter().map(|t| t.name.clone()).collect();
            let task_sel = dialoguer::FuzzySelect::new()
                .with_prompt("Select task")
                .items(&task_names)
                .default(0)
                .interact()?;
            (project.id, tasks[task_sel].id)
        }
    };

    let entry = client.start(pid, tid).await?;
    let projects = client.projects().await.unwrap_or_default();
    let project_name = projects
        .iter()
        .find(|p| p.id == entry.project_id)
        .map(|p| p.name.as_str())
        .unwrap_or("?");
    ui::success(&format!("Timer started — {}", project_name));
    Ok(())
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
        None => ui::step("No timer running"),
    }
    Ok(())
}

async fn cmd_stop() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    let entry = client.stop().await?;
    ui::success(&format!("Stopped — total {}", entry.elapsed_display()));
    Ok(())
}

async fn cmd_pause() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    client.pause().await?;
    ui::success("Timer paused");
    Ok(())
}

async fn cmd_resume() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    client.resume().await?;
    ui::success("Timer resumed");
    Ok(())
}

async fn cmd_upgrade() -> Result<()> {
    ui::step("Upgrading sablier");
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            "--git",
            "https://github.com/FacileStudio/sablier-cli.git",
            "--force",
        ])
        .status()?;
    if !status.success() {
        bail!("upgrade failed — run `cargo install --git https://github.com/FacileStudio/sablier-cli.git --force` manually");
    }
    ui::success("Upgraded to the latest version");
    Ok(())
}

async fn cmd_projects() -> Result<()> {
    let cfg = load_authed_config()?;
    let client = api::ApiClient::new(&cfg.server_url, &cfg.token);
    let projects = client.projects().await?;

    if projects.is_empty() {
        ui::step("No projects");
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
