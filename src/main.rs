use std::fs::DirEntry;
use std::thread;
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

#[derive(Clone)]
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

            for param in params {
                options
                    .ignore_directories
                    .push(param.to_string().to_lowercase())
            }
        } else if found_flag == "--a" {
            // allow folders
            let params = &flag[4..flag.len() - 1]; // Also trims '[]'

            let params = params.split(',');

            for param in params {
                options
                    .allow_directories
                    .push(param.to_string().to_lowercase())
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

    let result = find_files(&search, paths, options);

    let elapsed = now.elapsed();
    println!(
        "Searched {} files. Found {} files.",
        result.searched, result.found
    );
    println!("Completed in {:.2?}", elapsed)
}

fn find_files(search: &str, paths: ReadDir, options: Options) -> SearchResult {
    let mut result = SearchResult {
        found: 0,
        searched: 0,
    };

    let mut entries: Vec<DirEntry> = Vec::new();
    for path in paths {
        if let Result::Ok(entry) = path {
            entries.push(entry);
        }
    }

    let count = entries.len();

    if count >= 2 {
        let mid = count / 2;

        let mut first_half = Vec::new();
        let mut second_half = Vec::new();
        for (i, entry) in entries.into_iter().enumerate() {
            if i > mid {
                second_half.push(entry);
            } else {
                first_half.push(entry);
            }
        }

        let first_params = (search.to_owned(), options.clone());

        let handle_first = thread::spawn(move || {
            let search_result = search_entries(&first_params.0, first_half, first_params.1);
            result.searched += search_result.searched;
            result.found += search_result.found;
        });

        let second_params = (search.to_owned(), options.clone());

        let handle_second = thread::spawn(move || {
            let search_result = search_entries(&second_params.0, second_half, second_params.1);
            result.searched += search_result.searched;
            result.found += search_result.found;
        });

        handle_first.join().unwrap();
        handle_second.join().unwrap();
    } else {
        let search_result = search_entries(search, entries, options);
        result.searched += search_result.searched;
        result.found += search_result.found;
    }

    result
}

fn search_entries(search: &str, entries: Vec<DirEntry>, options: Options) -> SearchResult {
    let mut result = SearchResult {
        found: 0,
        searched: 0,
    };

    for entry in entries {
        let metadata = entry.metadata();
        let name = entry.file_name().into_string().unwrap_or_default();

        if let Result::Ok(data) = metadata {
            if data.is_dir() {
                let instant_allowed = options.root_only && !options.root;

                if !instant_allowed && options.ignore_directories.contains(&name.to_lowercase()) {
                    continue;
                }

                if !instant_allowed
                    && (options.allow_directories.len() > 0
                        && !options.allow_directories.contains(&name.to_lowercase()))
                {
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
                    let sub_res = find_files(search, sub_paths, sub_options);
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

    result
}

fn print_file(file: File) {
    println!("File: {}", file.name);
    println!("Path: {}", file.path.to_str().unwrap_or_default());
}
