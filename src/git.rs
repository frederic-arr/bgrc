use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::Cli;

pub fn check(args: &Cli, at: &Path) -> Result<()> {
    if !args.check.skip_worktree {
        has_no_uncommitted_files(args, at).context("worktree is not clean")?;
    }

    if !args.check.skip_stash {
        has_no_stashed_changes(args, at).context("has stashed changes")?;
    }

    if !args.check.offline && !args.check.skip_unpushed_commits {
        fetch(args, at).context("failed to fetch")?;
    }

    if !args.check.skip_unpushed_commits {
        has_no_unpushed_commits(args, at).context("has unpushed references")?;
    }

    Ok(())
}

fn cmd(args: &Cli, at: &Path) -> Command {
    let mut cmd = Command::new(&args.check.git_path);

    cmd.arg("-C").arg(at);

    cmd
}

fn has_no_uncommitted_files(args: &Cli, at: &Path) -> Result<()> {
    let r = cmd(args, at).arg("status").arg("--short").output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}

fn has_no_stashed_changes(args: &Cli, at: &Path) -> Result<()> {
    let r = cmd(args, at).arg("stash").arg("list").output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}

fn fetch(args: &Cli, at: &Path) -> Result<()> {
    let r = cmd(args, at).arg("fetch").arg("--all").output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}

fn has_no_unpushed_commits(args: &Cli, at: &Path) -> Result<()> {
    let r = cmd(args, at)
        .arg("log")
        .arg("--branches")
        .arg("--not")
        .arg("--remotes")
        .arg("--oneline")
        .output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}
