use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ai-teamlead")]
#[command(about = "Repo-local AI team lead CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Init,
    Poll,
    Run {
        #[arg(short = 'd', long = "debug")]
        debug: bool,
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
