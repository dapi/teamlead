use std::fs::OpenOptions;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub trait Shell {
    fn run(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<String>;
    fn run_with_env(
        &self,
        cwd: &Path,
        envs: &[(&str, &str)],
        program: &str,
        args: &[&str],
    ) -> Result<String>;
    fn spawn_with_env(
        &self,
        cwd: &Path,
        envs: &[(&str, &str)],
        program: &str,
        args: &[&str],
        stdout_stderr_log_path: Option<&Path>,
    ) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct SystemShell;

impl Shell for SystemShell {
    fn run(&self, cwd: &Path, program: &str, args: &[&str]) -> Result<String> {
        self.run_with_env(cwd, &[], program, args)
    }

    fn run_with_env(
        &self,
        cwd: &Path,
        envs: &[(&str, &str)],
        program: &str,
        args: &[&str],
    ) -> Result<String> {
        let output = Command::new(program)
            .args(args)
            .envs(envs.iter().copied())
            .current_dir(cwd)
            .output()
            .with_context(|| format!("failed to execute {program}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "command failed: {} {}: {}",
                program,
                args.join(" "),
                stderr.trim()
            );
        }

        let stdout = String::from_utf8(output.stdout)
            .with_context(|| format!("command output was not valid utf-8: {program}"))?;

        Ok(stdout.trim().to_string())
    }

    fn spawn_with_env(
        &self,
        cwd: &Path,
        envs: &[(&str, &str)],
        program: &str,
        args: &[&str],
        stdout_stderr_log_path: Option<&Path>,
    ) -> Result<()> {
        let mut command = Command::new(program);
        command
            .args(args)
            .envs(envs.iter().copied())
            .current_dir(cwd)
            .stdin(Stdio::null());

        if let Some(path) = stdout_stderr_log_path {
            let stdout = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .with_context(|| format!("failed to open spawn log: {}", path.display()))?;
            let stderr = stdout
                .try_clone()
                .with_context(|| format!("failed to clone spawn log handle: {}", path.display()))?;
            command
                .stdout(Stdio::from(stdout))
                .stderr(Stdio::from(stderr));
        } else {
            command.stdout(Stdio::null()).stderr(Stdio::null());
        }

        command
            .spawn()
            .with_context(|| format!("failed to spawn {program}"))?;
        Ok(())
    }
}
