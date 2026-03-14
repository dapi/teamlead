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
    Poll {
        #[arg(long = "zellij-session", value_name = "SESSION")]
        zellij_session: Option<String>,
    },
    Loop {
        #[arg(long = "zellij-session", value_name = "SESSION")]
        zellij_session: Option<String>,
    },
    Run {
        #[arg(short = 'd', long = "debug")]
        debug: bool,
        #[arg(long = "zellij-session", value_name = "SESSION")]
        zellij_session: Option<String>,
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
    BindZellijPane {
        session_uuid: String,
    },
    LaunchZellijFixture {
        issue: u64,
    },
    RenderLaunchAgentContext {
        issue: String,
    },
    CompleteStage {
        session_uuid: String,
        #[arg(long, value_enum)]
        outcome: crate::complete_stage::StageOutcome,
        #[arg(long)]
        message: String,
    },
}
