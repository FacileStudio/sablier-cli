use anyhow::Result;
use clap::{Subcommand, ValueEnum};

mod claude;
mod settings;

#[derive(Subcommand, Debug, Clone)]
pub enum HooksCommand {
    #[command(about = "Add the Sablier heartbeat hooks to an agent's settings")]
    Install {
        #[arg(value_enum)]
        target: Agent,
    },
    #[command(about = "Remove the Sablier heartbeat hooks, leaving every other hook")]
    Uninstall {
        #[arg(value_enum)]
        target: Agent,
    },
    #[command(about = "Print a hook snippet to paste into a tool's own hook file")]
    Print {
        #[arg(value_enum)]
        target: Snippet,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Agent {
    Claude,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Snippet {
    Git,
}

pub const GIT_POST_COMMIT: &str =
    "#!/bin/sh\nsablier heartbeat --source git --write >/dev/null 2>&1 &\n";

/// Dispatches `sablier hooks`.
pub fn run(cmd: HooksCommand) -> Result<()> {
    match cmd {
        HooksCommand::Install {
            target: Agent::Claude,
        } => claude::install(),
        HooksCommand::Uninstall {
            target: Agent::Claude,
        } => claude::uninstall(),
        HooksCommand::Print {
            target: Snippet::Git,
        } => {
            print!("{GIT_POST_COMMIT}");
            Ok(())
        }
    }
}
