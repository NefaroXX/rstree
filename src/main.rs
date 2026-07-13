//! `rstree` command-line interface.
//!
//! A lightweight clone of the Unix `tree` command. Given an optional directory
//! path (defaulting to the current directory), it recursively prints the
//! directory hierarchy using `tree` drawing characters. Standard library only.

use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;

use rstree::{generate, Options};

const HELP: &str = "\
rstree - a lightweight, dependency-free clone of the `tree` command

USAGE:
    rstree [OPTIONS] [PATH]

ARGS:
    [PATH]    Directory to display. Defaults to the current directory (.).

OPTIONS:
    -a, --all          Show hidden entries (names starting with '.').
    -L, --max-depth N  Descend at most N directory levels.
    -h, --help         Print this help message and exit.
    -V, --version      Print version information and exit.

EXAMPLES:
    rstree
    rstree ./src
    rstree -a -L 2 /etc
";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut show_hidden = false;
    let mut max_depth: Option<usize> = None;
    let mut path = ".".to_string();

    // Parse arguments. The first non-flag token is treated as the path.
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => show_hidden = true,
            "-L" | "--max-depth" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(d) => max_depth = Some(d),
                    None => {
                        eprintln!("Error: --max-depth requires a non-negative integer.");
                        process::exit(1);
                    }
                }
            }
            "-h" | "--help" => {
                print!("{}", HELP);
                return;
            }
            "-V" | "--version" => {
                println!("rstree {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            s if s.starts_with('-') && s != "-" => {
                eprintln!("Error: unknown option '{}'.", s);
                eprintln!("Try 'rstree --help' for usage.");
                process::exit(1);
            }
            s => path = s.to_string(),
        }
        i += 1;
    }

    let opts = Options {
        show_hidden,
        max_depth,
    };

    let root = Path::new(&path);
    let stdout = io::stdout();
    let mut out = stdout.lock();

    match generate(root, &mut out, &opts) {
        Ok(stats) => {
            let _ = writeln!(
                out,
                "\n{} directories, {} files",
                stats.directories, stats.files
            );
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("Error: '{}' does not exist.", path);
            process::exit(1);
        }
        Err(ref e) if e.kind() == io::ErrorKind::InvalidInput => {
            eprintln!("Error: '{}' is not a directory.", path);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: cannot access '{}': {}", path, e);
            process::exit(1);
        }
    }
}
