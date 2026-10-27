mod cli;
mod display;
mod git;
mod search;

use std::process::ExitCode;
use std::{io::IsTerminal, path::Path};

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use search::SearchResult;
use termcolor::{ColorChoice, StandardStream};

use crate::display::display;

#[derive(Debug)]
struct Outdated<'a> {
    repo: &'a Path,
    error: anyhow::Error,
}

fn main() -> Result<ExitCode> {
    let args = Cli::parse();

    let search_result = SearchResult::try_at(&args)?;
    let unversioned = search_result.unversioned();
    if args.check.fail_fast && !unversioned.is_empty() {
        todo!()
    }

    let mut ok = Vec::with_capacity(search_result.versioned().len());
    let mut nok = Vec::with_capacity(unversioned.len() + search_result.unknown().len());
    nok.extend(unversioned.iter().map(|v| NotOk {
        repo: v.to_path_buf(),
        error: CheckError::Unversioned,
    }));

    if !args.check.skip_git {
        for repo in search_result.versioned() {
            let check_result = git::check(&args, repo.as_path());
            let Err(error) = check_result else {
                ok.push(repo);
                continue;
            };

            outdated.push(Outdated { repo, error });
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
