use crate::print::*;

use std::path::PathBuf;

pub(crate) use clap::Parser;

/// Wordlists manipulation
///
/// WorMod takes a wordlist as input, manipulates it by applying the requested
/// operations and outputs the modified version.
/// The input wordlist shall contain one entry per line and will be treated as
/// if it does, which means that if one line contains multiple entries they
/// will be treated as if they were only one.
/// The input wordlist shall also contain only valid Unicode characters and the
/// process will exit-fail if it does not.
#[derive(Parser)]
#[command(about, version)]
pub(crate) struct Params {
    /// The path of the input wordlist
    ///
    /// Omit to read from stdin. In such a case, if not running in pipe mode,
    /// the whole input will be stored in memory and only flushed when the EOF
    /// is reached. Consider using --pipe if the input size is unknown: the
    /// process will eventually exit-fail in case the system will become close
    /// to run out of memory.
    #[arg(short, long, value_name="FILE")]
    pub(crate) input: Option<PathBuf>,
    /// The path of the output wordlist
    ///
    /// Omit to write to stdout.
    #[arg(short, long, value_name="FILE")]
    pub(crate) output: Option<PathBuf>,
    /// Append to the output file instead of overwriting
    ///
    /// Can only be used in conjunction with --output.
    #[arg(long, action=clap::ArgAction::SetTrue)]
    pub(crate) append_output: bool,
    /// Do not follow symlinks in input/output paths
    #[arg(long, action=clap::ArgAction::SetTrue)]
    pub(crate) no_follow_symlinks: bool,
    /// Keep reading and flushing instead of waiting for the EOF
    ///
    /// Reccomended when the input is stdin and the output is stdout, in order
    /// to reduce memory usage, or when the input size is unknown.
    /// In pipe mode the input is read one line at a time, manipulated
    /// on-the-fly and suddenly written to the output.
    /// Some operations are not available or may have downsides when running
    /// in pipe mode. See --sort and --unique for further details.
    #[arg(long, action=clap::ArgAction::SetTrue)]
    pub(crate) pipe: bool,
    /// Sort the wordlist
    ///
    /// Cannot be used in conjunction with --pipe.
    #[arg(long, action=clap::ArgAction::SetTrue)]
    pub(crate) sort: bool,
    /// Remove duplicates from the wordlist
    ///
    /// When used in conjunction with --pipe, in order to provide only unique
    /// entries an internal list of all the past entries will be kept. Memory
    /// usage will hence increase accordingly and the process will eventually
    /// exit-fail in case the system become close to run out of memory, not to
    /// mention the performance overhead of re-checking the entire list at each
    /// iteration.
    #[arg(long, action=clap::ArgAction::SetTrue)]
    pub(crate) unique: bool,
    /// Reverse each entry (not the wordlist itself)
    #[arg(long, action=clap::ArgAction::SetTrue)]
    pub(crate) reverse: bool,
    /// Discard entries shorter than the given length
    #[arg(long, value_name="N", action=clap::ArgAction::Set)]
    pub(crate) min_len: Option<usize>,
    /// Discard entries longer than the given length
    #[arg(long, value_name="N", action=clap::ArgAction::Set)]
    pub(crate) max_len: Option<usize>,
}

impl Params {
    /// Checks the options to ensure they are consistent
    pub(crate) fn validate(mut self) -> Self {
        self.validate_paths();
        self.validate_length_range();
        self.validate_operations();
        self
    }

    /// Checks the input and output paths to ensure they are consistent
    fn validate_paths(&mut self) {
        self.validate_input_path();
        self.validate_output_path();
        if let (Some(in_path), Some(out_path)) = (self.input.as_ref(), self.output.as_ref()) {
            if in_path == out_path {
                exit_err!(
                    ("Input and output paths resolve to the same resource: {:?}", in_path)
                );
            }
        }
    }

    /// Checks the intput path and canonicalizes it
    pub(crate) fn validate_input_path(&mut self) {
        if self.input.is_none() {
            return;
        }
        let p = self.input.clone().unwrap();
        if self.no_follow_symlinks && p.contains_symlinks() {
            exit_err!(
                ("Input path contains symlinks: {:?}", p)
            );
        }
        match p.canonicalize() {
            Err(e) => {
                exit_err!(
                    ("Failed to resolve input path: {:?}", p),
                    ("Failed to canonicalize: {}", e.to_string())
                );
            },
            Ok(path) => {
                match std::fs::exists(path.clone()) {
                    Err(e) => {
                        exit_err!(
                            ("Failed to validate input path: {:?}", path),
                            ("Error while checking for existence: {}", e.to_string())
                        );
                    },
                    Ok(false) => {
                        exit_err!(
                            ("Input wordlist not found at path: {:?}", p)
                        );
                    },
                    Ok(true) => {
                        if path.is_dir() {
                            exit_err!(
                                ("Input path is a directory: {:?}", p)
                            );
                        }
                        self.input = Some(path);
                    },
                }
            }
        }
    }

    /// Checks the output path and canonicalizes it
    pub(crate) fn validate_output_path(&mut self) {
        if self.output.is_none() {
            return;
        }
        let ref p = self.output.clone().unwrap();
        if self.no_follow_symlinks && p.contains_symlinks() {
            exit_err!(
                ("Output path contains symlinks: {:?}", p)
            );
        } else if p.is_dir() {
            exit_err!(
                ("Output path is a directory: {:?}", p)
            );
        }
        match std::fs::exists(p) {
            Err(e) => {
                exit_err!(
                    ("Failed to validate output path: {:?}", p),
                    ("Error while checking for existence: {}", e.to_string())
                );
            },
            Ok(true) => {
                self.output = std::fs::canonicalize(p)
                    .map_err(|e| {
                        exit_err!(
                            ("Failed to resolve output path: {:?}", p),
                            ("Failed to canonicalize: {}", e.to_string())
                        );
                    }).ok();
            },
            Ok(false) => match p.parent() {
                Some(dir) => {
                    let file = p.file_name().unwrap_or_else(|| {
                        exit_err!(
                            ("Failed to get file name in output path: {:?}", p)
                        );
                    });
                    self.output = std::fs::canonicalize(dir)
                        .map_err(|e| {
                            exit_err!(
                                ("Failed to resolve output path component: {:?}", dir),
                                ("Failed to canonicalize parent directory: {}", e.to_string())
                            );
                        }).map(|d| {
                            d.join(file)
                        }).ok();
                },
                None => {
                    exit_err!(
                        ("Unexpected output path: {:?}", p)
                    );
                }
            },
        }
    }

    /// Checks the length range to ensure it is consistent
    fn validate_length_range(&self) {
        match (self.min_len, self.max_len) {
            (Some(min), Some(max)) => {
                if max < min {
                    exit_err!(
                        ("Invalid min-max length values: {}-{}", min, max),
                        ("Maximum length cannot be smaller than minimum length")
                    );
                } else if min == usize::MAX {
                    exit_err!(
                        ("Invalid min length: {}", max),
                        ("This is equivalent to a no-op")
                    );
                } else if max == 0 {
                    exit_err!(
                        ("Invalid max length: {}", max),
                        ("This is equivalent to a no-op")
                    );
                }
            },
            (Some(min), None) => {
                if min == usize::MAX {
                    exit_err!(
                        ("Invalid min length: {}", min),
                        ("This is equivalent to a no-op")
                    );
                }
            },
            (None, Some(max)) => {
                if max == 0 {
                    exit_err!(
                        ("Invalid max length: {}", max),
                        ("This is equivalent to a no-op")
                    );
                }
            },
            (None, None) => (),
        }
    }

    /// Checks the scheduled operations to ensure they are consistent
    fn validate_operations(&self) {
        if !self.sort && !self.unique && !self.reverse && self.min_len.is_none() && self.max_len.is_none() {
            exit_err!(
                ("No manipulation option is set"),
                ("This is equivalent to a no-op")
            );
        } else if self.output.is_none() && self.append_output {
            exit_err!(
                ("Incompatible option: --append-output"),
                ("Cannot use append to a file without an output file")
            );
        } else if self.output.is_none() && self.append_output {
            exit_err!(
                ("Incompatible option: --append-output"),
                ("Cannot append to a file without an output file")
            );
        } else if self.pipe && self.sort {
            exit_err!(
                ("Incompatible options: --pipe --sort"),
                ("Cannot sort a pipe flow")
            );
        }
    }

    /// Repeats the checks on the input path to try to ensure consistency
    pub(crate) fn check_input_path(&self) {
        if let Some(p) = self.input.as_ref() {
            if !p.exists() {
                exit_err!(
                    ("Input wordlist not found at path: {:?}", p)
                );
            } else if p.is_dir() {
                exit_err!(
                    ("Input path is a directory: {:?}", p)
                );
            } else if self.no_follow_symlinks && p.contains_symlinks() {
                exit_err!(
                    ("Input path contains symlinks: {:?}", p)
                );
            }
        }
    }

    /// Repeats the checks on the output path to try to ensure consistency
    pub(crate) fn check_output_path(&self) {
        if let Some(p) = self.output.as_ref() {
            if p.is_dir() {
                exit_err!(
                    ("Output path is a directory: {:?}", p)
                );
            } else if self.no_follow_symlinks && p.contains_symlinks() {
                exit_err!(
                    ("Output path contains symlinks: {:?}", p)
                );
            }
        }
    }

    /// Whether the entries shall be filtered by length
    pub(crate) fn has_length_range(&self) -> bool {
        self.min_len.is_some() || self.max_len.is_some()
    }
}

trait PathOps {
    /// Checks all the components of a path to spot symlinks
    fn contains_symlinks(&self) -> bool;
}

impl PathOps for PathBuf {
    fn contains_symlinks(&self) -> bool {
        let mut path = PathBuf::new();
        for component in self.components() {
            path.push(component);
            match std::fs::symlink_metadata(&path) {
                Err(e) => {
                    exit_err!(
                        ("Failed to validate output path component: {:?}", path),
                        ("Failed to check symlink: {}", e.to_string())
                    );
                },
                Ok(md) => {
                    if md.is_symlink() {
                        return true;
                    }
                },
            }
        }
        false
    }
}
