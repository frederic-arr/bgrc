mod cli;
mod display;
mod git;
mod search;

use std::path::Path;
use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use search::SearchResult;
use termcolor::{ColorChoice, StandardStream};

use crate::display::display;

#[derive(Debug)]
struct Outdated<'a> {
    repo: &'a Path,
    error: anyhow::Error,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let search_result = SearchResult::try_at(&args)?;
    let unversioned = search_result.unversioned();
    if args.fail_fast && !unversioned.is_empty() {
        todo!()
    }

    let mut ok = Vec::with_capacity(search_result.versioned().len());
    let mut outdated = Vec::with_capacity(search_result.versioned().len());

    if args.git {
        for repo in search_result.versioned() {
            let check_result = git::check(&args, repo.as_path());
            let Err(error) = check_result else {
                ok.push(repo);
                continue;
            };

            outdated.push(Outdated { repo, error });
            if args.fail_fast {
                break;
            }
        }
    }

    let color_choice = match args.color {
        cli::Color::Auto => ColorChoice::Auto,
        cli::Color::Never => ColorChoice::Never,
        cli::Color::Always => ColorChoice::Always,
        cli::Color::Ansi => ColorChoice::AlwaysAnsi,
    };
    let mut stdout = StandardStream::stdout(color_choice);
    let mut stderr = StandardStream::stderr(color_choice);

    display(
        &mut stdout,
        &mut stderr,
        &args,
        &search_result,
        &ok,
        &outdated,
    )?;

    if (search_result.unversioned().len() + outdated.len()) == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}
