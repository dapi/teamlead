use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::shell::Shell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoContext {
    pub repo_root: PathBuf,
    pub git_dir: PathBuf,
    pub github_owner: String,
    pub github_repo: String,
}

impl RepoContext {
    pub fn discover(shell: &dyn Shell, cwd: &Path) -> Result<Self> {
        let repo_root = PathBuf::from(shell.run(cwd, "git", &["rev-parse", "--show-toplevel"])?);
        let git_dir = PathBuf::from(shell.run(cwd, "git", &["rev-parse", "--git-dir"])?);
        let origin = shell.run(cwd, "git", &["remote", "get-url", "origin"])?;
        let remote = RemoteSlug::parse(&origin)?;

        let git_dir = if git_dir.is_relative() {
            repo_root.join(git_dir)
        } else {
            git_dir
        };

        Ok(Self {
            repo_root,
            git_dir,
            github_owner: remote.owner,
            github_repo: remote.repo,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteSlug {
    owner: String,
    repo: String,
}

impl RemoteSlug {
    fn parse(input: &str) -> Result<Self> {
        let value = input.trim();
        let slug = if let Some(rest) = value.strip_prefix("git@github.com:") {
            rest
        } else if let Some(rest) = value.strip_prefix("https://github.com/") {
            rest
        } else if let Some(rest) = value.strip_prefix("ssh://git@github.com/") {
            rest
        } else {
            bail!("unsupported git remote url: {value}");
        };

        let slug = slug.trim_end_matches(".git");
        let mut parts = slug.split('/');
        let owner = parts.next().context("remote slug is missing owner")?;
        let repo = parts.next().context("remote slug is missing repo")?;
        if parts.next().is_some() {
            bail!("remote slug contains unexpected extra path segments: {value}");
        }

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteSlug;

    #[test]
    fn parses_ssh_remote() {
        let remote = RemoteSlug::parse("git@github.com:dapi/teamlead.git").expect("ssh remote");
        assert_eq!(remote.owner, "dapi");
        assert_eq!(remote.repo, "teamlead");
    }

    #[test]
    fn parses_https_remote() {
        let remote = RemoteSlug::parse("https://github.com/dapi/teamlead").expect("https remote");
        assert_eq!(remote.owner, "dapi");
        assert_eq!(remote.repo, "teamlead");
    }
}
