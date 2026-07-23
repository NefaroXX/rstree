use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;

use ls_tree::{generate, Options, SortField};

const HELP: &str = "\
ls-tree - a lightweight clone of the `tree` command

USAGE:
    ls-tree [OPTIONS] [PATH]

ARGS:
    [PATH]    Directory to display. Defaults to the current directory (.).

OPTIONS:
    -a, --all             Show hidden entries (names starting with '.').
    -L, --max-depth N     Descend at most N directory levels.
    --dirs-only           Display directories only.
    -s, --size            Show file sizes.
    -h, --human-readable  Print sizes in human-readable format (implies --size).
    --json                Output JSON instead of the classic tree.
    --git-ignore          Respect .gitignore rules.
    --sort=<field>        Sort entries by name (default), size, or time.
    --prune               Omit empty directories from the output.
    --icons               Show Unicode file-type icons.
    --total-size          Show recursive directory sizes.
    --color               Force colour output (auto-detected by default).
    --no-color            Disable colour output.
    --help                Print this help message and exit.
    -V, --version         Print version information and exit.

EXAMPLES:
    ls-tree
    ls-tree ./src
    ls-tree -a -L 2 /etc
    ls-tree --json --size ./src
    ls-tree --git-ignore --sort=size
";

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::default();
    let mut path = ".".to_string();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-a" | "--all" => opts.show_hidden = true,
            "-L" | "--max-depth" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse::<usize>().ok()) {
                    Some(d) => opts.max_depth = Some(d),
                    None => {
                        eprintln!("Error: --max-depth requires a non-negative integer.");
                        process::exit(1);
                    }
                }
            }
            "--dirs-only" => opts.dirs_only = true,
            "-s" | "--size" => opts.show_size = true,
            "-h" | "--human-readable" => {
                opts.human_readable = true;
                opts.show_size = true;
            }
            "--json" => opts.json = true,
            "--git-ignore" => opts.git_ignore = true,
            "--sort" => {
                i += 1;
                match args.get(i).map(|s| s.as_str()) {
                    Some("name") => opts.sort_by = SortField::Name,
                    Some("size") => opts.sort_by = SortField::Size,
                    Some("time") => opts.sort_by = SortField::Time,
                    Some(other) => {
                        eprintln!(
                            "Error: --sort requires 'name', 'size', or 'time', got '{}'.",
                            other
                        );
                        process::exit(1);
                    }
                    None => {
                        eprintln!("Error: --sort requires a value (name/size/time).");
                        process::exit(1);
                    }
                }
            }
            "--prune" => opts.prune = true,
            "--icons" => opts.show_icons = true,
            "--total-size" => {
                opts.show_total_size = true;
                opts.show_size = true;
            }
            "--color" => opts.color = true,
            "--no-color" => opts.color = false,
            "--help" => {
                print!("{}", HELP);
                return;
            }
            "-V" | "--version" => {
                println!("ls-tree {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            s if s.starts_with('-') && s != "-" => {
                eprintln!("Error: unknown option '{}'.", s);
                eprintln!("Try 'ls-tree --help' for usage.");
                process::exit(1);
            }
            s => path = s.to_string(),
        }
        i += 1;
    }

    let root = Path::new(&path);
    let stdout = io::stdout();
    let mut out = stdout.lock();

    match generate(root, &mut out, &opts) {
        Ok(stats) => {
            if !opts.json {
                let _ = writeln!(
                    out,
                    "\n{} directories, {} files",
                    stats.directories, stats.files
                );
            }
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
