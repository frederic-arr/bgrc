use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
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
        unknown: Vec<PathBuf>,
    },
    Versioned {
        versioned: Vec<PathBuf>,
    },
    Unknown {
        unknown: Vec<PathBuf>,
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

        SearchResult::recursive_search(args, &mut matcher, &args.path)
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

    pub fn unknown(&self) -> &[PathBuf] {
        match self {
            SearchResult::Unknown { unknown } | SearchResult::Mixed { unknown, .. } => {
                unknown.as_slice()
            }
            SearchResult::Versioned { .. }
            | SearchResult::Unversioned { .. }
            | SearchResult::Empty => &[],
        }
    }

    fn recursive_search(
        args: &Cli,
        matcher: &mut IncrementalIgnore,
        path: &Path,
    ) -> Result<SearchResult> {
        let mut versioned = vec![];
        let mut unversioned = vec![];
        let mut unknown: Vec<PathBuf> = vec![];

        let Ok(iter) = std::fs::read_dir(path) else {
            return Ok(SearchResult::Unknown {
                unknown: vec![path.to_path_buf()],
            });
        };

        for entry in iter {
            if args.check.fail_fast && !unversioned.is_empty() {
                return Ok(SearchResult::Unversioned {
                    unversioned: vec![path.to_path_buf()],
                });
            }

            let Ok(entry) = entry else {
                unknown.push(path.to_path_buf());
                continue;
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
                unknown.push(path);
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

            let Ok(git_folder_exists) = std::fs::exists(path.join(".git")) else {
                unknown.push(path);
                continue;
            };

            if git_folder_exists {
                versioned.push(path);
                continue;
            }

            let Ok(result) = SearchResult::recursive_search(args, matcher, &path) else {
                unknown.push(path);
                continue;
            };

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
            (0, 0, 0) => Ok(SearchResult::Empty),
            (0, 0, _) => Ok(SearchResult::Unknown {
                unknown: vec![path.to_path_buf()],
            }),
            (0, _, 0) => Ok(SearchResult::Unversioned {
                unversioned: vec![path.to_path_buf()],
            }),
            (_, 0, 0) => Ok(SearchResult::Versioned { versioned }),
            (_, _, _) => Ok(SearchResult::Mixed {
                versioned,
                unversioned,
                unknown,
            }),
        }
    }
}
