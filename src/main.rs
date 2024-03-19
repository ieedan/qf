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

#[derive(Debug)]
struct SearchResult {
    found: u32,
    searched: u32,
}

#[derive(Clone, Debug)]
struct DirList {
    directories: Vec<String>,
    root_only: bool,
}

impl DirList {
    fn new() -> Self {
        DirList {
            directories: Vec::new(),
            root_only: false
        }
    }
}

#[derive(Clone, Debug)]
struct Options {
    ignore: DirList,
    allow: DirList,
    // Will run non-concurrently when true
    disable_concurrency: bool,
    // True when the parent directory is the working directory
    root: bool,
    // The minimum directories to allow threads to be spawned must be minimum 2
    min_concurrent_directories: i32,
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

const FLAGS: [&str; 6] = ["--i", "--ri", "--a", "--ra", "--dc", "--c"];

fn main() {
    let now = Instant::now();

    let search = env::args().nth(1).expect("Please provide a search value.");
    let flags = env::args().skip(2);

    let mut options = Options {
        ignore: DirList::new(),
        allow: DirList::new(),
        disable_concurrency: false,
        min_concurrent_directories: 2,
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
            panic!("invalid argument '{flag}' found");
        }

        if found_flag == "--i" {
            // ignore folders
            let params = &flag[4..flag.len() - 1]; // Also trims '[]'

            let params = params.split(',');

            for param in params {
                options
                    .ignore.directories
                    .push(param.to_string().to_lowercase())
            }
        } else if found_flag == "--a" {
            // allow folders
            let params = &flag[4..flag.len() - 1]; // Also trims '[]'

            let params = params.split(',');

            for param in params {
                options
                    .allow.directories
                    .push(param.to_string().to_lowercase())
            }
        } else if found_flag == "--c" {
            let param = &flag[4..flag.len() - 1]; // Also trims '[]'

            let msg = format!("{param} is not a valid value for the --c flag");

            let value = param.parse::<i32>().expect(&msg);

            if value < 2 {
                panic!("The value for --c: {param} is too small. The minimum value is 2.");
            }

            options.min_concurrent_directories = value;
        } else if found_flag == "--ra" {
            options.allow.root_only = true;
        } else if found_flag == "--ri" {
            options.ignore.root_only = true;
        } else if found_flag == "--dc" {
            options.disable_concurrency = true;
        }
    }

    let root = env::current_dir().expect("Could not determine working directory.");

    let paths = fs::read_dir(&root).expect("No files found in the working directory.");

    println!("Searching for {search}...");

    if options.ignore.directories.len() > 0 {
        println!("Ignoring {:?}", options.ignore.directories);
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

    if count < 4 || options.disable_concurrency {
        let search_result = search_entries(search, entries, options.clone());
        result.searched += search_result.searched;
        result.found += search_result.found;
    } else {
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
            return search_entries(&first_params.0, first_half, first_params.1);
        });

        let second_params = (search.to_owned(), options.clone());

        let handle_second = thread::spawn(move || {
            return search_entries(&second_params.0, second_half, second_params.1);
        });

        let first_res = handle_first.join().unwrap();
        let second_res = handle_second.join().unwrap();
        result.searched += first_res.searched + second_res.searched;
        result.found += first_res.found + second_res.found;
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
                let instant_allowed = options.ignore.root_only && !options.root;

                if !instant_allowed && options.ignore.directories.contains(&name.to_lowercase()) {
                    continue;
                }

                let instant_allowed = options.allow.root_only && !options.root;

                if !instant_allowed
                    && (options.allow.directories.len() > 0
                        && !options.allow.directories.contains(&name.to_lowercase()))
                {
                    continue;
                }

                let sub_paths = fs::read_dir(entry.path());
                if let Result::Ok(sub_paths) = sub_paths {
                    let sub_options = Options {
                        root: false, // Set this to false after first iteration
                        ignore: options.ignore.clone(),
                        allow: options.allow.clone(),
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
