mod cli;
mod display;
mod git;
mod search;

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use search::SearchResult;
use termcolor::{ColorChoice, StandardStream};

use crate::display::display;
use crate::git::CheckError;

#[derive(Debug)]
struct NotOk {
    repo: PathBuf,
    error: CheckError,
}

fn main() -> Result<ExitCode> {
    let args = Cli::parse();

    let search_result = SearchResult::try_at(&args)?;
    let unversioned = search_result.unversioned();
    let unknown = search_result.unknown();

    let mut ok = Vec::with_capacity(search_result.versioned().len());
    let mut nok = Vec::with_capacity(unversioned.len() + search_result.unknown().len());
    nok.extend(unversioned.iter().map(|v| NotOk {
        repo: v.to_path_buf(),
        error: CheckError::Unversioned,
    }));

    nok.extend(unknown.iter().map(|v| NotOk {
        repo: v.to_path_buf(),
        error: CheckError::Unknown,
    }));

    if !args.check.skip_git
        && !(args.check.fail_fast && (!unversioned.is_empty() || !unknown.is_empty()))
    {
        for repo in search_result.versioned() {
            let check_result = git::check(&args, repo.as_path());
            let Err(error) = check_result else {
                ok.push(repo);
                continue;
            };

            nok.push(NotOk {
                repo: repo.to_path_buf(),
                error,
            });
            if args.check.fail_fast {
                break;
            }
        }
    }

    if args.output.sort {
        ok.sort();
        nok.sort_by(|a, b| a.repo.cmp(&b.repo));
    }

    let color_choice = match args.output.color {
        cli::Color::Auto if !std::io::stdin().is_terminal() => ColorChoice::Never,
        cli::Color::Auto => ColorChoice::Auto,
        cli::Color::Never => ColorChoice::Never,
        cli::Color::Always => ColorChoice::Always,
        cli::Color::Ansi => ColorChoice::AlwaysAnsi,
    };
    let mut stdout = StandardStream::stdout(color_choice);
    let mut stderr = StandardStream::stderr(color_choice);

    display(&mut stdout, &mut stderr, &args, &ok, &nok)?;

    if (search_result.unversioned().len() + nok.len()) == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}
