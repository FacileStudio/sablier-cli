use anyhow::Result;
use clap::{Args, Subcommand};

use crate::api::{ApiClient, CreateKeyRequest, Key};
use crate::ui;

#[derive(Subcommand, Debug, Clone)]
pub enum KeysCommand {
    #[command(about = "List registered API keys")]
    List(KeysListArgs),
    #[command(about = "Create a new API key")]
    Create(KeysCreateArgs),
    #[command(about = "Revoke an API key")]
    Revoke(KeysRevokeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct KeysListArgs {
    #[arg(short, long, help = "Filter keys by application name")]
    pub app: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct KeysCreateArgs {
    #[arg(short, long, help = "Application name")]
    pub app: String,

    #[arg(long, help = "Create a public browser key instead of a secret key")]
    pub public: bool,

    #[arg(long, help = "Comma-separated allowed origins (for public keys)")]
    pub origins: Option<String>,

    #[arg(long, help = "Daily event quota limit (for public keys)")]
    pub quota: Option<i64>,
}

#[derive(Args, Debug, Clone)]
pub struct KeysRevokeArgs {
    #[arg(help = "Key ID to revoke")]
    pub id: i64,

    #[arg(short, long, help = "Confirm revocation without prompting")]
    pub yes: bool,
}

pub async fn run(cmd: KeysCommand, client: &ApiClient, json: bool) -> Result<()> {
    match cmd {
        KeysCommand::List(args) => list(client, args, json).await,
        KeysCommand::Create(args) => create(client, args, json).await,
        KeysCommand::Revoke(args) => revoke(client, args, json).await,
    }
}

async fn list(client: &ApiClient, args: KeysListArgs, json: bool) -> Result<()> {
    let mut keys = client.list_keys(args.app.as_deref()).await?;
    if let Some(ref a) = args.app {
        keys.retain(|k| &k.app == a);
    }
    if json {
        println!("{}", serde_json::to_string(&keys)?);
        return Ok(());
    }
    if keys.is_empty() {
        ui::step("no API keys found");
        return Ok(());
    }
    println!(
        "{:<6} {:<16} {:<8} {:<12} {:<8} {:<24} {}",
        "ID", "APP", "KIND", "PREFIX", "STATUS", "QUOTA", "CREATED"
    );
    for k in &keys {
        print_key_row(k);
    }
    Ok(())
}

fn print_key_row(k: &Key) {
    let status = if k.revoked_at.is_some() {
        "revoked"
    } else {
        "active"
    };
    let quota = if k.daily_quota > 0 {
        format!("{}/day ({} used)", k.daily_quota, k.used_today.unwrap_or(0))
    } else {
        "unlimited".to_string()
    };
    let created = if k.created_at.len() >= 10 {
        &k.created_at[..10]
    } else {
        &k.created_at
    };
    println!(
        "#{:<5} {:<16} {:<8} {:<12} {:<8} {:<24} {}",
        k.id, k.app, k.kind, k.prefix, status, quota, created
    );
}

async fn create(client: &ApiClient, args: KeysCreateArgs, json: bool) -> Result<()> {
    let kind = if args.public {
        "public".to_string()
    } else {
        "secret".to_string()
    };
    let origins = args
        .origins
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();

    let req = CreateKeyRequest {
        app: args.app,
        kind,
        allowed_origins: origins,
        daily_quota: args.quota,
    };
    let resp = client.create_key(&req).await?;
    if json {
        println!("{}", serde_json::to_string(&resp)?);
    } else {
        ui::success(&format!(
            "created {} key for {} (id: {})",
            resp.key.kind, resp.key.app, resp.key.id
        ));
        println!("{}", resp.token);
        ui::hint("store this token securely; it will not be shown again");
    }
    Ok(())
}

async fn revoke(client: &ApiClient, args: KeysRevokeArgs, json: bool) -> Result<()> {
    if !args.yes && !json {
        let prompt = format!("Revoke API key #{}?", args.id);
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(prompt)
            .default(false)
            .interact()?;
        if !confirmed {
            ui::step("aborted");
            return Ok(());
        }
    }
    client.revoke_key(args.id).await?;
    if json {
        println!("{}", serde_json::json!({ "revoked": args.id }));
    } else {
        ui::success(&format!("revoked key {}", args.id));
    }
    Ok(())
}
