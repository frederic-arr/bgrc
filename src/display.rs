use std::io::Write;
use std::path::PathBuf;

use crate::Outdated;
use crate::cli::{Cli, Format};
use crate::search::SearchResult;
use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

pub fn display(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    match args.output.format {
        Format::Std => display_std(stdout, stderr, args, results, ok, outdated),
        Format::Human => display_human(stdout, stderr, args, results, ok, outdated),
        Format::Json => display_json(stdout, stderr, args, results, ok, outdated),
        Format::Count => display_count(stdout, stderr, args, results, ok, outdated),
        Format::CountVersioned => {
            display_count_versioned(stdout, stderr, args, results, ok, outdated)
        }
        Format::CountUnversioned => {
            display_count_unversioned(stdout, stderr, args, results, ok, outdated)
        }
        Format::CountOutdated => {
            display_count_outdated(stdout, stderr, args, results, ok, outdated)
        }
        Format::CountUpToDate => {
            display_count_up_to_date(stdout, stderr, args, results, ok, outdated)
        }
    }
}

fn display_std(
    _stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    if args.output.show_versioned && !ok.is_empty() {
        for p in ok {
            println!("{}", p.display());
        }
    }

    if !args.output.skip_outdated && !outdated.is_empty() {
        for o in outdated {
            eprintln!("{}", o.repo.display());
        }
    }

    if !args.output.skip_unversioned && !results.unversioned().is_empty() {
        for p in results.unversioned() {
            eprintln!("{}", p.display());
        }
    }

    Ok(())
}

fn display_human(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    if !ok.is_empty() && args.output.show_versioned {
        if !args.output.skip_headings {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            writeln!(stdout, "== OK")?;
            stdout.reset()?;
        }

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(false))?;
        for p in ok {
            writeln!(stdout, "{}", p.display())?;
        }
        stdout.reset()?;

        if (!outdated.is_empty() && !args.output.skip_outdated)
            || (!results.unversioned().is_empty() && !args.output.skip_unversioned)
        {
            writeln!(stdout)?;
        }
    }

    if !outdated.is_empty() && !args.output.skip_outdated {
        if !args.output.skip_headings {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            writeln!(stdout, "== Outdated")?;
            stdout.reset()?;
        }

        for o in outdated {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
            if args.output.all_to_stdout {
                write!(stdout, "{}", o.repo.display())?;
            } else {
                write!(stderr, "{}", o.repo.display())?;
            }

            stdout.reset()?;
            stderr.reset()?;
            if args.output.all_to_stdout {
                writeln!(stdout, ": {}", o.error)?;
            } else {
                writeln!(stderr, ": {}", o.error)?;
            }
        }
        stdout.reset()?;
        stderr.reset()?;

        if !results.unversioned().is_empty() && !args.output.skip_unversioned {
            writeln!(stdout)?;
        }
    }

    if !results.unversioned().is_empty() && !args.output.skip_unversioned {
        if !args.output.skip_headings {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            writeln!(stdout, "== Unversioned")?;
            stdout.reset()?;
        }

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
        for p in results.unversioned() {
            if args.output.all_to_stdout {
                writeln!(stdout, "{}", p.display())?;
            } else {
                writeln!(stderr, "{}", p.display())?;
            }
        }
        stdout.reset()?;
        stderr.reset()?;
    }

    Ok(())
}

fn display_json(
    stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    _args: &Cli,
    results: &SearchResult,
    ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    writeln!(
        stdout,
        "{}",
        serde_json::to_string(&json!({
            "ok": ok,
            "outdated": outdated,
            "unversioned": results.unversioned(),
        }))?
    )?;

    Ok(())
}

fn display_count(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    _ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    let count = results.unversioned().len() + outdated.len();
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
    results: &SearchResult,
    _ok: &[&PathBuf],
    _outdated: &[Outdated],
) -> Result<()> {
    let count = results.versioned().len();
    writeln!(stdout, "{count}")?;

    Ok(())
}

fn display_count_unversioned(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    _ok: &[&PathBuf],
    _outdated: &[Outdated],
) -> Result<()> {
    let count = results.unversioned().len();
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
    _results: &SearchResult,
    _ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    let count = outdated.len();
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
    _results: &SearchResult,
    ok: &[&PathBuf],
    _outdated: &[Outdated],
) -> Result<()> {
    let count = ok.len();
    writeln!(stdout, "{count}")?;

    Ok(())
}

impl Serialize for Outdated<'_> {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.repo.serialize(serializer)
    }
}
