mod cli;
mod git;
mod search;

use anyhow::{Result, anyhow};
use clap::Parser;
use cli::{Args, Color, Format};
use search::SearchResult;

fn main() -> Result<()> {
    let args = Args::parse();

    let search_result = SearchResult::try_at(&args)?;
    dbg!(&search_result);
    dbg!(&args);

    let unversioned = search_result.unversioned();
    if args.fail_fast && !unversioned.is_empty() {
        todo!()
    }

    let mut ok = Vec::with_capacity(search_result.versioned().len());
    let mut outdated = Vec::with_capacity(search_result.versioned().len());

    if args.git {
        for repo in search_result.versioned() {
            let check_result = git::check(&args, repo.as_path());

            match check_result {
                Ok(_) => ok.push(repo),
                Err(_) => outdated.push(repo),
            }

            if args.fail_fast && check_result.is_err() {
                break;
            }
        }
    }

    dbg!(ok);
    dbg!(outdated);

    Ok(())
}
