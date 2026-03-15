use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::agent_flow::{AgentFlowAgent, AgentFlowMode};
use crate::config::LaunchTarget;
use crate::domain::FlowStage;

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
        #[arg(long = "launch-target", value_name = "TARGET", value_enum)]
        launch_target: Option<LaunchTarget>,
        issue: String,
    },
    #[command(hide = true)]
    Internal {
        #[command(subcommand)]
        internal: InternalCommand,
    },
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use crate::config::LaunchTarget;
    use clap::Parser;

    #[test]
    fn parses_run_launch_target_pane() {
        let cli = Cli::try_parse_from(["ai-teamlead", "run", "42", "--launch-target", "pane"])
            .expect("cli should parse");
        let Command::Run {
            launch_target,
            issue,
            ..
        } = cli.command
        else {
            panic!("expected run command");
        };
        assert_eq!(issue, "42");
        assert_eq!(launch_target, Some(LaunchTarget::Pane));
    }

    #[test]
    fn parses_run_launch_target_tab() {
        let cli = Cli::try_parse_from(["ai-teamlead", "run", "42", "--launch-target", "tab"])
            .expect("cli should parse");
        let Command::Run { launch_target, .. } = cli.command else {
            panic!("expected run command");
        };
        assert_eq!(launch_target, Some(LaunchTarget::Tab));
    }

    #[test]
    fn poll_does_not_accept_launch_target_override() {
        let error = Cli::try_parse_from(["ai-teamlead", "poll", "--launch-target", "pane"])
            .expect_err("poll must reject launch target override");
        assert!(error.to_string().contains("--launch-target"));
    }

    #[test]
    fn loop_does_not_accept_launch_target_override() {
        let error = Cli::try_parse_from(["ai-teamlead", "loop", "--launch-target", "tab"])
            .expect_err("loop must reject launch target override");
        assert!(error.to_string().contains("--launch-target"));
    }
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
        #[arg(long, value_enum, default_value = "analysis")]
        stage: FlowStage,
        #[arg(long, value_enum)]
        outcome: crate::complete_stage::StageOutcome,
        #[arg(long)]
        message: String,
    },
}
