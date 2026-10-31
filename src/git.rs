// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Frédéric ARROYO <opensource@frederic-arroyo.ch>

use std::path::Path;
use std::process::Command;

use anyhow::{Result, ensure};
use thiserror::Error;

use crate::Cli;

#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum CheckError {
    #[error("Unclean worktree")]
    UncleanWorktree,

    #[error("Stashed changes")]
    StashedChanges,

    #[error("Fetch failure")]
    FetchFailure,

    #[error("Unpushed references")]
    UnpushedReferences,

    #[error("Unversioned")]
    Unversioned,

    #[error("{0}")]
    Unknown(String),
}

pub fn check(args: &Cli, at: &Path) -> std::result::Result<(), CheckError> {
    if !args.check.skip_worktree {
        has_no_uncommitted_files(args, at).map_err(|_| CheckError::UncleanWorktree)?;
    }

    if !args.check.skip_stash {
        has_no_stashed_changes(args, at).map_err(|_| CheckError::StashedChanges)?;
    }

    if !args.check.offline && !args.check.skip_unpushed_commits {
        fetch(args, at).map_err(|_| CheckError::FetchFailure)?;
    }

    if !args.check.skip_unpushed_commits {
        has_no_unpushed_commits(args, at).map_err(|_| CheckError::UnpushedReferences)?;
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
