use std::{fmt::Display, path::PathBuf};

use clap::{ArgAction, Args, Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    pub path: PathBuf,

    #[command(flatten, next_help_heading = "Search options")]
    pub search: SearchArgs,

    #[command(flatten, next_help_heading = "Check options")]
    pub check: CheckArgs,

    #[command(flatten, next_help_heading = "Output options")]
    pub output: OutputArgs,
}

#[derive(Args, Debug)]
pub struct SearchArgs {
    #[arg(short, long, default_value_t = 5)]
    pub depth: usize,

    #[arg(short, long)]
    pub ignore: Vec<String>,

    #[arg(long, default_value_t = false)]
    pub use_common_filters: bool,

    #[arg(long, default_value_t = false)]
    pub no_require_git: bool,

    #[arg(long, default_value_t = false)]
    pub no_hidden: bool,

    #[arg(long, default_value_t = false)]
    pub no_parents: bool,

    #[arg(long, default_value_t = false)]
    pub no_ignore: bool,

    #[arg(long, default_value_t = false)]
    pub no_git_ignore: bool,

    #[arg(long, default_value_t = false)]
    pub no_git_global: bool,

    #[arg(long, default_value_t = false)]
    pub no_git_exclude: bool,

    #[arg(long, default_value_t = false)]
    pub no_follow_link: bool,

    #[arg(long)]
    pub ignore_file: Vec<PathBuf>,

    #[arg(long, default_value_t = false)]
    pub no_ignore_file_case_insensitive: bool,

    #[arg(long, default_value_t = false)]
    pub one_file_system: bool,
}

#[derive(Args, Debug)]
pub struct CheckArgs {
    #[arg(short = 'g', long, default_value_t = false)]
    pub skip_git: bool,

    #[arg(long, default_value = "git")]
    pub git_path: PathBuf,

    #[arg(long, default_value_t = false)]
    pub offline: bool,

    #[arg(long, default_value_t = false)]
    pub skip_stash: bool,

    #[arg(long, default_value_t = false)]
    pub skip_worktree: bool,

    #[arg(long, default_value_t = false)]
    pub skip_unpushed_commits: bool,

    #[arg(long, default_value_t = false)]
    pub fail_fast: bool,
}

#[derive(Args, Debug)]
pub struct OutputArgs {
    #[arg(long, default_value_t = false)]
    pub show_versioned: bool,

    #[arg(long, default_value_t = true, action = ArgAction::Set)]
    pub show_outdated: bool,

    #[arg(long, default_value_t = false)]
    pub show_unversioned: bool,

    #[arg(long, default_value_t)]
    pub color: Color,

    #[arg(long, default_value_t)]
    pub format: Format,

    #[arg(long, default_value_t = false)]
    pub all_to_stdout: bool,

    #[arg(long, default_value_t = false)]
    pub no_new_line: bool,

    #[arg(long, default_value_t = false)]
    pub quiet: bool,
}

#[derive(Debug, Default, Clone, ValueEnum)]
pub enum Color {
    #[default]
    Auto,
    Never,
    Always,
    Ansi,
}

#[derive(Debug, Default, Clone, ValueEnum)]
pub enum Format {
    Std,
    #[default]
    Human,
    Json,
    Count,
    CountVersioned,
    CountUnversioned,
    CountOutdated,
    CountUpToDate,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Auto => f.write_str("auto"),
            Color::Never => f.write_str("never"),
            Color::Always => f.write_str("always"),
            Color::Ansi => f.write_str("ansi"),
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Std => f.write_str("std"),
            Format::Human => f.write_str("human"),
            Format::Json => f.write_str("json"),
            Format::Count => f.write_str("count"),
            Format::CountVersioned => f.write_str("count-versioned"),
            Format::CountUnversioned => f.write_str("count-unversioned"),
            Format::CountOutdated => f.write_str("count-outdated"),
            Format::CountUpToDate => f.write_str("count-up-to-date"),
        }
    }
}
