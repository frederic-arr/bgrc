// SPDX-License-Identifier: MIT OR Apache-2.0
// SPDX-FileCopyrightText: 2026 Frédéric ARROYO <opensource@frederic-arroyo.ch>

use std::path::{Path, PathBuf};

use anyhow::{Context, Error, Result, anyhow};
use ignore::{IncrementalIgnore, WalkBuilder, overrides::OverrideBuilder};

use crate::Cli;

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
        unknown: Vec<(PathBuf, PathBuf, Error)>,
    },
    Versioned {
        versioned: Vec<PathBuf>,
    },
    Unknown {
        unknown: Vec<(PathBuf, PathBuf, Error)>,
    },
    Empty,
}

impl SearchResult {
    pub fn try_at(args: &Cli) -> Result<SearchResult> {
        let mut overrides = &mut OverrideBuilder::new(&args.path);
        if args.search.use_common_filters {
            for path in COMMON_EXCLUDE {
                overrides = overrides.add(&format!("!{path}").to_string())?;
            }
        }

        for glob in &args.search.ignore {
            overrides = overrides.add(&format!("!{{{glob}}}"))?;
        }

        let mut builder = WalkBuilder::new(&args.path);
        let builder = builder
            .max_depth(Some(args.search.depth))
            .hidden(!args.search.no_hidden)
            .parents(!args.search.no_parents)
            .ignore(!args.search.no_ignore)
            .git_ignore(!args.search.no_git_ignore)
            .git_global(!args.search.no_git_global)
            .git_exclude(!args.search.no_git_exclude)
            .follow_links(!args.search.no_follow_link)
            .require_git(!args.search.no_require_git)
            .ignore_case_insensitive(!args.search.no_ignore_file_case_insensitive)
            .same_file_system(args.search.one_file_system)
            .overrides(overrides.build()?);

        for path in &args.search.ignore_file {
            builder.add_ignore(path).map_or(Ok(()), Err)?;
        }

        let matcher = builder.build_matchers();
        let mut matcher = matcher
            .first()
            .ok_or(anyhow!("a matcher was expected"))?
            .clone();

        Ok(SearchResult::recursive_search(
            args,
            &mut matcher,
            &args.path,
        ))
    }

    pub fn versioned(&self) -> &[PathBuf] {
        match self {
            SearchResult::Versioned { versioned } | SearchResult::Mixed { versioned, .. } => {
                versioned.as_slice()
            }
            SearchResult::Unversioned { .. }
            | SearchResult::Unknown { .. }
            | SearchResult::Empty => &[],
        }
    }

    pub fn unversioned(&self) -> &[PathBuf] {
        match self {
            SearchResult::Unversioned { unversioned } | SearchResult::Mixed { unversioned, .. } => {
                unversioned.as_slice()
            }
            SearchResult::Versioned { .. } | SearchResult::Unknown { .. } | SearchResult::Empty => {
                &[]
            }
        }
    }

    pub fn unknown(&self) -> &[(PathBuf, PathBuf, Error)] {
        match self {
            SearchResult::Unknown { unknown } | SearchResult::Mixed { unknown, .. } => {
                unknown.as_slice()
            }
            SearchResult::Versioned { .. }
            | SearchResult::Unversioned { .. }
            | SearchResult::Empty => &[],
        }
    }

    fn recursive_search(args: &Cli, matcher: &mut IncrementalIgnore, root: &Path) -> SearchResult {
        let mut versioned = vec![];
        let mut unversioned = vec![];
        let mut unknown: Vec<(PathBuf, PathBuf, Error)> = vec![];

        let iter = match std::fs::read_dir(root).context("unable to read directory") {
            Ok(v) => v,
            Err(err) => {
                return SearchResult::Unknown {
                    unknown: vec![(root.to_path_buf(), root.to_path_buf(), err)],
                };
            }
        };

        for entry in iter {
            if args.check.fail_fast && !unversioned.is_empty() {
                return SearchResult::Unversioned {
                    unversioned: vec![root.to_path_buf()],
                };
            }

            let entry = match entry.context("unable to read content of directory") {
                Ok(v) => v,
                Err(err) => {
                    unknown.push((root.to_path_buf(), root.to_path_buf(), err));
                    continue;
                }
            };

            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                if !args.check.skip_unversioned {
                    unversioned.push(path);
                }
                continue;
            };

            if !file_type.is_dir() {
                if !args.check.skip_unversioned {
                    unversioned.push(path);
                }
                continue;
            }

            let Some(normalized) = matcher.normalize(&path) else {
                unknown.push((
                    root.to_path_buf(),
                    path,
                    anyhow!("unable to normalize path"),
                ));
                continue;
            };

            if matcher.matched(&normalized, true).is_ignore() {
                continue;
            }

            if !matcher.matched(normalized, true).should_descend() {
                if !args.check.skip_unversioned {
                    unversioned.push(path);
                }
                continue;
            }

            let git_folder_exists = match std::fs::exists(path.join(".git"))
                .context("unable to check for git folder")
            {
                Ok(v) => v,
                Err(err) => {
                    unknown.push((root.to_path_buf(), path.to_path_buf(), err));
                    continue;
                }
            };

            if git_folder_exists {
                versioned.push(path);
                continue;
            }

            let result = SearchResult::recursive_search(args, matcher, &path);

            match result {
                SearchResult::Unversioned { unversioned: mut u } => {
                    if !args.check.skip_unversioned {
                        unversioned.append(&mut u)
                    }
                }
                SearchResult::Mixed {
                    versioned: mut v,
                    unversioned: mut u,
                    unknown: mut w,
                } => {
                    if !args.check.skip_unversioned {
                        unversioned.append(&mut u);
                    }
                    versioned.append(&mut v);
                    unknown.append(&mut w);
                }
                SearchResult::Versioned { versioned: mut v } => versioned.append(&mut v),
                SearchResult::Unknown { unknown: mut w } => unknown.append(&mut w),
                SearchResult::Empty => {}
            }
        }

        match (versioned.len(), unversioned.len(), unknown.len()) {
            (0, 0, 0) => SearchResult::Empty,
            (0, 0, 1) => {
                for u in unknown.iter_mut() {
                    u.0 = root.to_path_buf()
                }
                SearchResult::Unknown { unknown }
            }
            (0, 0, _) => {
                unknown.push((
                    root.to_path_buf(),
                    root.to_path_buf(),
                    anyhow!("unable to verify directory"),
                ));
                SearchResult::Unknown { unknown }
            }
            (0, _, 0) => SearchResult::Unversioned {
                unversioned: vec![root.to_path_buf()],
            },
            (_, 0, 0) => SearchResult::Versioned { versioned },
            (_, _, _) => SearchResult::Mixed {
                versioned,
                unversioned,
                unknown,
            },
        }
    }
}
