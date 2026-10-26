use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, ensure};

use crate::Args;

pub fn check(args: &Args, at: &Path) -> Result<()> {
    if !args.skip_worktree {
        has_no_uncommitted_files(args, at).context("worktree is not clean")?;
    }

    if !args.skip_stash {
        has_no_stashed_changes(args, at).context("has stashed changes")?;
    }

    if !args.offline && !args.skip_unpushed_commits {
        fetch(args, at).context("failed to fetch")?;
    }

    if !args.skip_unpushed_commits {
        has_no_unpushed_commits(args, at).context("has unpushed references")?;
    }

    Ok(())
}

fn cmd(args: &Args, at: &Path) -> Command {
    let mut cmd = Command::new(&args.git_path);

    cmd.arg("-C").arg(at);

    cmd
}

fn has_no_uncommitted_files(args: &Args, at: &Path) -> Result<()> {
    let r = cmd(args, at).arg("status").arg("--short").output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}

fn has_no_stashed_changes(args: &Args, at: &Path) -> Result<()> {
    let r = cmd(args, at).arg("stash").arg("list").output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}

fn fetch(args: &Args, at: &Path) -> Result<()> {
    let r = cmd(args, at).arg("fetch").arg("--all").output()?;

    ensure!(r.status.success());
    ensure!(r.stdout.is_empty());
    ensure!(r.stderr.is_empty());

    Ok(())
}

fn has_no_unpushed_commits(args: &Args, at: &Path) -> Result<()> {
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
