use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::agent_flow::{AgentFlowAgent, AgentFlowMode};

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
    Test {
        #[command(subcommand)]
        test: TestCommand,
    },
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
pub enum TestCommand {
    AgentFlow(TestAgentFlowArgs),
}

#[derive(Debug, Args)]
pub struct TestAgentFlowArgs {
    #[arg(long = "scenario", value_name = "NAME")]
    pub scenario: String,
    #[arg(long = "agent", value_enum)]
    pub agent: Option<AgentFlowAgent>,
    #[arg(long = "mode", value_enum)]
    pub mode: Option<AgentFlowMode>,
    #[arg(long = "keep-sandbox")]
    pub keep_sandbox: bool,
    #[arg(long = "artifacts-dir", value_name = "PATH")]
    pub artifacts_dir: Option<PathBuf>,
    #[arg(long = "timeout-seconds", value_name = "SECONDS")]
    pub timeout_seconds: Option<u64>,
    #[arg(long = "no-build")]
    pub no_build: bool,
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
