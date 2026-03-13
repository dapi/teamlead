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
    Poll,
    Run { issue: String },
}
