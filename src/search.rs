use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use ignore::{IncrementalIgnore, WalkBuilder, overrides::OverrideBuilder};

use crate::Args;

const COMMON_EXCLUDE: [&str; 9] = [
    "node_modules",
    "target",
    "build",
    "dist",
    "out",
    ".cache",
    "cache",
    "venv",
    ".venv",
];

#[derive(Debug)]
pub enum SearchResult {
    Unversioned {
        unversioned: Vec<PathBuf>,
    },
    Mixed {
        versioned: Vec<PathBuf>,
        unversioned: Vec<PathBuf>,
    },
    Versioned {
        versioned: Vec<PathBuf>,
    },
    Empty,
}

impl SearchResult {
    pub fn try_at(args: &Args) -> Result<SearchResult> {
        let mut overrides = &mut OverrideBuilder::new(&args.path);
        if args.use_common_filters {
            for path in COMMON_EXCLUDE {
                overrides = overrides.add(&format!("!{path}").to_string())?;
            }
        }

        for glob in &args.ignore {
            overrides = overrides.add(&format!("!{{{glob}}}")).unwrap();
        }

        let mut builder = WalkBuilder::new(&args.path);
        let builder = builder
            .max_depth(Some(args.depth))
            .hidden(!args.no_hidden)
            .parents(!args.no_parents)
            .ignore(!args.no_ignore)
            .git_ignore(!args.no_git_ignore)
            .git_global(!args.no_git_global)
            .git_exclude(!args.no_git_exclude)
            .follow_links(!args.no_follow_link)
            .require_git(!args.no_require_git)
            .ignore_case_insensitive(!args.no_ignore_file_case_insensitive)
            .same_file_system(args.one_file_system)
            .overrides(overrides.build()?);

        for path in &args.ignore_file {
            builder.add_ignore(path).map_or(Ok(()), Err)?;
        }

        let matcher = builder.build_matchers();
        let mut matcher = matcher
            .first()
            .ok_or(anyhow!("a matcher was expected"))?
            .clone();

        SearchResult::recursive_search(args, &mut matcher, &args.path)
    }

    pub fn versioned(&self) -> &[PathBuf] {
        match self {
            SearchResult::Versioned { versioned } | SearchResult::Mixed { versioned, .. } => {
                versioned.as_slice()
            }
            SearchResult::Unversioned { .. } | SearchResult::Empty => &[],
        }
    }

    pub fn unversioned(&self) -> &[PathBuf] {
        match self {
            SearchResult::Unversioned { unversioned } | SearchResult::Mixed { unversioned, .. } => {
                unversioned.as_slice()
            }
            SearchResult::Versioned { .. } | SearchResult::Empty => &[],
        }
    }

    fn recursive_search(
        args: &Args,
        matcher: &mut IncrementalIgnore,
        path: &Path,
    ) -> Result<SearchResult> {
        let mut versioned = vec![];
        let mut unversioned = vec![];

        for entry in std::fs::read_dir(path)? {
            if args.fail_fast && !unversioned.is_empty() {
                return Ok(SearchResult::Unversioned {
                    unversioned: vec![path.to_path_buf()],
                });
            }

            let entry = entry.unwrap();
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                unversioned.push(path);
                continue;
            };

            if !file_type.is_dir() {
                unversioned.push(path);
                continue;
            }

            let normalized = matcher.normalize(&path).unwrap();
            if matcher.matched(&normalized, true).is_ignore() {
                continue;
            }

            if !matcher.matched(normalized, true).should_descend() {
                unversioned.push(path);
                continue;
            }

            if std::fs::exists(path.join(".git")).is_ok_and(|v| v) {
                versioned.push(path);
                continue;
            }

            match SearchResult::recursive_search(args, matcher, &path)? {
                SearchResult::Unversioned { unversioned: mut u } => unversioned.append(&mut u),
                SearchResult::Mixed {
                    versioned: mut v,
                    unversioned: mut u,
                } => {
                    unversioned.append(&mut u);
                    versioned.append(&mut v);
                }
                SearchResult::Versioned { versioned: mut v } => versioned.append(&mut v),
                SearchResult::Empty => {}
            }
        }

        match (versioned.len(), unversioned.len()) {
            (0, 0) => Ok(SearchResult::Empty),
            (0, _) => Ok(SearchResult::Unversioned {
                unversioned: vec![path.to_path_buf()],
            }),
            (_, 0) => Ok(SearchResult::Versioned { versioned }),
            (_, _) => Ok(SearchResult::Mixed {
                versioned,
                unversioned,
            }),
        }
    }
}
