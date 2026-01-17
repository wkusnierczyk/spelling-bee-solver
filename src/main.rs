//! CLI entry point for Spelling Bee Solver.

use clap::Parser;
use sbs::{Config, Solver}; // Removed unused SbsError
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "sbs")]
#[command(version = "0.1.2")]
#[command(disable_version_flag = true)]
#[command(about = "Spelling Bee Solver tool", long_about = None)]
struct Args {
    /// Available letters (e.g., "abcdefg")
    #[arg(short, long)]
    letters: Option<String>,

    /// Obligatory letter(s) (e.g., "a")
    #[arg(short, long)]
    present: Option<String>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Path to dictionary file (overrides config)
    #[arg(short, long)]
    dictionary: Option<PathBuf>,

    /// Output file path (if omitted, prints to stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Display developer information
    #[arg(long)]
    about: bool,
}

fn print_about() {
    println!("sbs: Spelling Bee Solver tool");
    println!("├─ version:   0.1.2");
    println!("├─ developer: mailto:waclaw.kusnierczyk@gmail.com");
    println!("├─ source:    https://github.com/wkusnierczyk/ips-sampler");
    println!("├─ licence:   MIT https://opensource.org/licenses/MIT");
    println!("└─ usage:     sbs --help");
}

fn main() {
    let args = Args::parse();

    if args.about {
        print_about();
        return;
    }

    // 1. Load Config from file or default
    let mut config = if let Some(path) = args.config {
        match Config::from_file(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error loading config file: {}", e);
                process::exit(1);
            }
        }
    } else {
        Config::default()
    };

    // 2. Override with CLI args
    if let Some(l) = args.letters {
        config.letters = Some(l);
    }
    if let Some(p) = args.present {
        config.present = Some(p);
    }
    if let Some(d) = args.dictionary {
        config.dictionary = d;
    }
    if let Some(o) = args.output {
        config.output = Some(o);
    }

    // 3. Validate minimal inputs
    if config.letters.is_none() || config.present.is_none() {
        eprintln!("Error: 'letters' and 'present' (obligatory) letters are required.");
        eprintln!("Provide them via --letters/-l and --present/-p or a config file.");
        process::exit(1);
    }

    // 4. Initialize Solver
    let mut solver = Solver::new(config.clone()); // Clone config for solver ownership

    // 5. Load Dictionary
    if let Err(e) = solver.load_dictionary() {
        eprintln!("Error loading dictionary: {}", e);
        eprintln!("Tip: Ensure you have run 'make setup' to download the default dictionary.");
        process::exit(1);
    }

    // 6. Solve
    match solver.solve() {
        Ok(words) => {
            let mut sorted_words: Vec<_> = words.into_iter().collect();
            sorted_words.sort();

            // 7. Output handling
            if let Some(out_path) = config.output {
                let path = PathBuf::from(out_path);
                match File::create(&path) {
                    Ok(mut file) => {
                        for word in sorted_words {
                            if let Err(e) = writeln!(file, "{}", word) {
                                eprintln!("Error writing to output file: {}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creating output file {:?}: {}", path, e);
                        process::exit(1);
                    }
                }
            } else {
                // Print to Stdout
                for word in sorted_words {
                    println!("{}", word);
                }
            }
        }
        Err(e) => {
            eprintln!("Error solving puzzle: {}", e);
            process::exit(1);
        }
    }
}
