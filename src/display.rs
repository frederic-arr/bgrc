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

    if args.output.show_outdated && !outdated.is_empty() {
        for o in outdated {
            eprintln!("{}", o.repo.display());
        }
    }

    if args.output.show_unversioned && !results.unversioned().is_empty() {
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
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        writeln!(stdout, "== OK")?;
        stdout.reset()?;

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(false))?;
        for p in ok {
            printa(stdout, stderr, args, &format!("{}", p.display()))?;
        }
        stdout.reset()?;

        if (!outdated.is_empty() && args.output.show_outdated)
            || (!results.unversioned().is_empty() && args.output.show_unversioned)
        {
            writeln!(stdout)?;
        }
    }

    if !outdated.is_empty() && args.output.show_outdated {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        writeln!(stdout, "== Outdated")?;

        for o in outdated {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
            stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
            eprint(stdout, stderr, args, &format!("{}", o.repo.display()))?;

            stdout.reset()?;
            stderr.reset()?;
            eprintln(stdout, stderr, args, &format!(": {}", o.error))?;
        }
        stdout.reset()?;
        stderr.reset()?;

        if !results.unversioned().is_empty() && args.output.show_unversioned {
            writeln!(stdout)?;
        }
    }

    if !results.unversioned().is_empty() && args.output.show_unversioned {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        writeln!(stdout, "== Unversioned")?;

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
        stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(false))?;
        for p in results.unversioned() {
            eprinta(stdout, stderr, args, &format!("{}", p.display()))?;
        }
        stdout.reset()?;
        stderr.reset()?;
    }

    Ok(())
}

fn display_json(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    ok: &[&PathBuf],
    outdated: &[Outdated],
) -> Result<()> {
    printa(
        stdout,
        stderr,
        args,
        &serde_json::to_string(&json!({
            "ok": ok,
            "outdated": outdated,
            "unversioned": results.unversioned(),
        }))
        .unwrap(),
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
    eprinta(stdout, stderr, args, &format!("{count}"))?;

    Ok(())
}

fn display_count_versioned(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    results: &SearchResult,
    _ok: &[&PathBuf],
    _outdated: &[Outdated],
) -> Result<()> {
    let count = results.versioned().len();
    printa(stdout, stderr, args, &format!("{count}"))?;

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
    eprinta(stdout, stderr, args, &format!("{count}"))?;

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
    eprinta(stdout, stderr, args, &format!("{count}"))?;

    Ok(())
}

fn display_count_up_to_date(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    _results: &SearchResult,
    ok: &[&PathBuf],
    _outdated: &[Outdated],
) -> Result<()> {
    let count = ok.len();
    printa(stdout, stderr, args, &format!("{count}"))?;

    Ok(())
}

fn print(
    stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    _args: &Cli,
    s: &str,
) -> Result<(), std::io::Error> {
    write!(stdout, "{s}")
}

fn println(
    stdout: &mut StandardStream,
    _stderr: &mut StandardStream,
    _args: &Cli,
    s: &str,
) -> Result<(), std::io::Error> {
    writeln!(stdout, "{s}")
}

fn eprint(
    _stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    _args: &Cli,
    s: &str,
) -> Result<(), std::io::Error> {
    write!(stderr, "{s}")
}

fn eprintln(
    _stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    _args: &Cli,
    s: &str,
) -> Result<(), std::io::Error> {
    writeln!(stderr, "{s}")
}

fn printa(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    s: &str,
) -> Result<(), std::io::Error> {
    if args.output.no_new_line {
        print(stdout, stderr, args, s)
    } else {
        println(stdout, stderr, args, s)
    }
}

fn eprinta(
    stdout: &mut StandardStream,
    stderr: &mut StandardStream,
    args: &Cli,
    s: &str,
) -> Result<(), std::io::Error> {
    if args.output.all_to_stdout {
        printa(stdout, stderr, args, s)
    } else {
        if args.output.no_new_line {
            eprint(stdout, stderr, args, s)
        } else {
            eprintln(stdout, stderr, args, s)
        }
    }
}

impl Serialize for Outdated<'_> {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.repo.serialize(serializer)
    }
}
