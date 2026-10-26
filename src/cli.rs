use std::path::PathBuf;

use clap::{ArgAction, Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub path: PathBuf,

    // Search options
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

    // Check options
    #[arg(short, long, default_value_t = true, action = ArgAction::Set)]
    pub git: bool,

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

    // Output options
    #[arg(long, default_value_t = true, action = ArgAction::Set)]
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
    #[default]
    Auto,
    Std,
    Human,
    Json,
    Count,
    CountVersioned,
    CountUnversioned,
    CountOutdated,
    CountUpToDate,
}

impl ToString for Color {
    fn to_string(&self) -> String {
        match self {
            Color::Auto => "auto".to_string(),
            Color::Never => "never".to_string(),
            Color::Always => "always".to_string(),
            Color::Ansi => "ansi".to_string(),
        }
    }
}

impl ToString for Format {
    fn to_string(&self) -> String {
        match self {
            Format::Auto => "auto".to_string(),
            Format::Std => "std".to_string(),
            Format::Human => "human".to_string(),
            Format::Json => "json".to_string(),
            Format::Count => "count".to_string(),
            Format::CountVersioned => "count-versioned".to_string(),
            Format::CountUnversioned => "count-unversioned".to_string(),
            Format::CountOutdated => "count-outdated".to_string(),
            Format::CountUpToDate => "count-up-to-date".to_string(),
        }
    }
}
