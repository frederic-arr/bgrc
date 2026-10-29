use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use crate::NotOk;
use crate::cli::{Cli, Format};
use crate::git::CheckError;
use anyhow::Result;
use serde::Serialize;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub fn display(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    match args.output.format {
        Format::Std => display_std(stdout, stderr, args, ok, ko),
        Format::Human => display_human(stdout, stderr, args, ok, ko),
        Format::Json => display_json(stdout, stderr, args, ok, ko),
        Format::Count => display_count(stdout, stderr, args, ok, ko),
        Format::CountVersioned => display_count_versioned(stdout, stderr, args, ok, ko),
        Format::CountUnversioned => display_count_unversioned(stdout, stderr, args, ok, ko),
        Format::CountOutdated => display_count_outdated(stdout, stderr, args, ok, ko),
        Format::CountUpToDate => display_count_up_to_date(stdout, stderr, args, ok, ko),
    }
}

fn display_std(
    _stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    args: &Cli,
    ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    if args.output.show_ok {
        for p in ok {
            println!("{}", p.display());
        }
    }

    if !args.output.skip_outdated {
        for o in ko {
            eprintln!("{}", o.repo.display());
        }
    }

    Ok(())
}

fn create_groups<'a>(
    args: &Cli,
    ko: &'a [NotOk],
) -> (usize, Vec<(&'static str, CheckError, Vec<&'a NotOk>)>) {
    let mut groups = Vec::with_capacity(10);
    groups.push(("unknown", CheckError::Unknown(String::new())));

    if !args.check.skip_unversioned {
        groups.push(("unversioned", CheckError::Unversioned));
    }

    if !args.check.skip_stash {
        groups.push(("stashed_changes", CheckError::StashedChanges));
    }

    if !args.check.skip_worktree {
        groups.push(("stashed_changes", CheckError::UncleanWorktree));
    }

    if !args.check.offline {
        groups.push(("stashed_changes", CheckError::FetchFailure));
    }

    if !args.check.skip_unpushed_commits {
        groups.push(("stashed_changes", CheckError::UnpushedReferences));
    }

    let mut output_groups = Vec::with_capacity(groups.len());
    let mut total = 0;
    for (k, group) in &mut groups {
        let values = if let CheckError::Unknown(_) = group {
            ko.iter()
                .filter(|v| matches!(v.error, CheckError::Unknown(_)))
                .collect::<Vec<_>>()
        } else {
            ko.iter().filter(|v| v.error == *group).collect::<Vec<_>>()
        };
        total += values.len();

        if values.is_empty() && args.output.hide_empty {
            continue;
        }

        output_groups.push((*k, group.clone(), values));
    }

    (total, output_groups)
}

fn display_group<'a>(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    heading: &str,
    values: &[&NotOk],
    show_error: bool,
) -> Result<()> {
    if values.is_empty() {
        return Ok(());
    }

    if !args.output.hide_headings {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        writeln!(stdout, "== {}", heading)?;
        stdout.reset()?;
    }

    let stream = if args.output.all_to_stdout {
        stdout
    } else {
        stderr
    };

    for v in values {
        stream.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
        write!(stream, "{}", v.repo.display())?;
        stream.reset()?;

        if show_error {
            write!(stream, ": {:#}", v.error)?;
        }

        if show_error && let Some(root) = &v.root {
            write!(stream, " in ")?;
            stream.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(false))?;
            write!(stream, "{}", root.display())?;
            stream.reset()?;
        }

        writeln!(stream, "")?;
    }

    Ok(())
}

fn display_human(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    if !ok.is_empty() && args.output.show_ok {
        if !args.output.hide_headings {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            writeln!(stdout, "== Ok")?;
            stdout.reset()?;
        }

        for v in ok {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(false))?;
            writeln!(stdout, "{}", v.display())?;
            stdout.reset()?;
        }
    }

    if args.output.group {
        let (mut remaining, groups) = create_groups(args, ko);
        for (_, group, values) in groups {
            if values.is_empty() {
                continue;
            }

            if matches!(group, CheckError::Unknown(_)) {
                display_group(stdout, stderr, args, "Other", values.as_slice(), true)?;
            } else {
                display_group(
                    stdout,
                    stderr,
                    args,
                    &group.to_string(),
                    values.as_slice(),
                    false,
                )?;
            }

            remaining -= values.len();
            if remaining > 0 {
                writeln!(stdout, "")?;
            }
        }
    } else {
        let v = ko.iter().collect::<Vec<_>>();
        display_group(stdout, stderr, args, "Not ok", &v, true)?;
    }

    if ko.len() == 0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)))?;
        writeln!(stdout, "All good, everything is versioned.")?;
        stdout.reset()?;
    }

    Ok(())
}

fn display_json(
    stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    args: &Cli,
    ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    let mut data = HashMap::new();
    if args.output.show_ok {
        data.insert("ok", ok.iter().map(|v| v.to_path_buf()).collect::<Vec<_>>());
    }

    if args.output.group {
        let (_, groups) = create_groups(args, ko);
        for (k, _, values) in groups {
            if values.is_empty() {
                continue;
            }

            data.insert(k, values.iter().map(|v| v.repo.to_path_buf()).collect());
        }
    } else {
        if ko.is_empty() && args.output.hide_empty {
        } else {
            data.insert(
                "ko",
                ko.iter().map(|v| v.repo.to_path_buf()).collect::<Vec<_>>(),
            );
        }
    }
    write!(stdout, "{}", serde_json::to_string(&data)?)?;

    if !args.output.no_new_line {
        writeln!(stdout, "")?;
    }

    Ok(())
}

fn display_count(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    _ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    let count = ko.len();
    if args.output.all_to_stdout {
        writeln!(stdout, "{count}")?;
    } else {
        writeln!(stderr, "{count}")?;
    }

    Ok(())
}

fn display_count_versioned(
    stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    _args: &Cli,
    ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    let count = ok.len()
        + ko.iter()
            .filter(|v| v.error != CheckError::Unversioned)
            .count();
    writeln!(stdout, "{count}")?;

    Ok(())
}

fn display_count_unversioned(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    _ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    let count = ko
        .iter()
        .filter(|v| v.error == CheckError::Unversioned)
        .count();

    if args.output.all_to_stdout {
        writeln!(stdout, "{count}")?;
    } else {
        writeln!(stderr, "{count}")?;
    }

    Ok(())
}

fn display_count_outdated(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    _ok: &[&PathBuf],
    ko: &[NotOk],
) -> Result<()> {
    let count = ko
        .iter()
        .filter(|v| v.error != CheckError::Unversioned)
        .count();

    if args.output.all_to_stdout {
        writeln!(stdout, "{count}")?;
    } else {
        writeln!(stderr, "{count}")?;
    }

    Ok(())
}

fn display_count_up_to_date(
    stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    _args: &Cli,
    ok: &[&PathBuf],
    _outdated: &[NotOk],
) -> Result<()> {
    let count = ok.len();
    writeln!(stdout, "{count}")?;

    Ok(())
}

impl Serialize for NotOk {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.repo.serialize(serializer)
    }
}
