use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ai-teamlead")]
#[command(about = "Repo-local AI team lead daemon")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Daemon,
    Init,
    Poll,
    Run {
        issue: String,
    },
    #[command(hide = true)]
    Internal {
        #[command(subcommand)]
        internal: InternalCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum InternalCommand {
    BindZellijPane { session_uuid: String },
    LaunchZellijFixture { issue: u64 },
    RenderLaunchAgentContext { issue: String },
}
