use std::time::Instant;
use std::{
    env,
    fs::{self, ReadDir},
    path::PathBuf,
};

struct File {
    name: String,
    path: PathBuf,
}

struct SearchResult {
    found: u32,
    searched: u32,
}

struct Options {
    // Skips searching these directories
    ignore_directories: Vec<String>,
    // Only searches these directories
    allow_directories: Vec<String>,
    // When enabled allowed directories only apply root directory
    root_only: bool,
    // True when the parent directory is the working directory
    root: bool,
}

trait PatternMatch {
    fn matches_pattern(&self, pattern: &str) -> bool;
}

impl PatternMatch for String {
    fn matches_pattern(&self, pattern: &str) -> bool {
        let trimmed_pattern = &pattern.replace("*", "");
        if pattern.starts_with(WILD_CARD) {
            if pattern.ends_with(WILD_CARD) {
                if self.contains(trimmed_pattern) {
                    return true;
                }
            } else {
                if self.ends_with(trimmed_pattern) {
                    return true;
                }
            }
        } else if pattern.ends_with(WILD_CARD) {
            if self.starts_with(trimmed_pattern) {
                return true;
            }
        } else {
            if trimmed_pattern == self {
                return true;
            }
        }

        false
    }
}

const WILD_CARD: char = '*';

const FLAGS: [&str; 3] = ["--i", "--a", "--r"];

fn main() {
    let now = Instant::now();

    let search = env::args().nth(1).expect("Please provide a search value.");
    let flags = env::args().skip(2);

    let mut options = Options {
        ignore_directories: vec![],
        allow_directories: vec![],
        root_only: false,
        root: true,
    };

    for (_, flag) in flags.into_iter().enumerate() {
        let mut found_flag = "";
        for (_, f) in FLAGS.iter().enumerate() {
            if flag.starts_with(f) {
                found_flag = f;
            }
        }

        if found_flag == "" {
            println!("qf does not provide a definition for the flag provided {flag}");
            continue;
        }

        if found_flag == "--i" {
            // ignore folders
            let params = &flag[4..flag.len() - 1]; // Also trims '[]'

            let params = params.split(',');

            for (_, param) in params.into_iter().enumerate() {
                options.ignore_directories.push(param.to_string().to_lowercase())
            }
        } else if found_flag == "--a" {
            // allow folders
            let params = &flag[4..flag.len() - 1]; // Also trims '[]'

            let params = params.split(',');

            for (_, param) in params.into_iter().enumerate() {
                options.allow_directories.push(param.to_string().to_lowercase())
            }
        } else if found_flag == "--r" {
            options.root_only = true;
        }
    }

    let root = env::current_dir().expect("Could not determine working directory.");

    let paths = fs::read_dir(&root).expect("No files found in the working directory.");

    println!("Searching for {search}...");

    if options.ignore_directories.len() > 0 {
        println!("Ignoring {:?}", options.ignore_directories);
    }

    let result = find(&search, paths, options);

    let elapsed = now.elapsed();
    println!("Searched {} files. Found {} files.", result.searched, result.found);
    println!("Completed in {:.2?}", elapsed)
}

fn find(search: &str, paths: ReadDir, options: Options) -> SearchResult {
    let mut result = SearchResult {
        found: 0,
        searched: 0,
    };

    for (_, path) in paths.into_iter().enumerate() {
        if let Result::Ok(entry) = path {
            let metadata = entry.metadata();
            let name = entry.file_name().into_string().unwrap_or_default();

            if let Result::Ok(data) = metadata {
                if data.is_dir() {
                    if options.ignore_directories.contains(&name.to_lowercase()) {
                        continue;
                    }

                    // If there are no rules or the rules only apply to the root and we are not at the root
                    // then we continue to search this directory
                    let instant_allowed = (options.root_only && !options.root)
                        || options.allow_directories.len() == 0;

                    if !instant_allowed && !options.allow_directories.contains(&name.to_lowercase()) {
                        continue;
                    }

                    let sub_paths = fs::read_dir(entry.path());
                    if let Result::Ok(sub_paths) = sub_paths {
                        let sub_options = Options {
                            root: false, // Set this to false after first iteration
                            ignore_directories: options.ignore_directories.clone(),
                            allow_directories: options.allow_directories.clone(),
                            ..options
                        };
                        let sub_res = find(search, sub_paths, sub_options);
                        result.found += sub_res.found;
                        result.searched += sub_res.searched;
                    }
                } else {
                    result.searched += 1;
                    if name.to_lowercase().matches_pattern(&search.to_lowercase()) {
                        result.found += 1;
                        print_file(File {
                            name,
                            path: entry.path(),
                        });
                    }
                }
            }
        }
    }

    result
}

fn print_file(file: File) {
    println!("File: {}", file.name);
    println!("Path: {}", file.path.to_str().unwrap_or_default());
}
