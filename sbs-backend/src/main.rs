//! CLI entry point for Spelling Bee Solver.

use clap::Parser;
use sbs::{create_validator, Config, Dictionary, Solver, ValidatorKind};
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
    #[arg(short, long)]
    letters: Option<String>,
    #[arg(short, long)]
    present: Option<String>,
    #[arg(short, long)]
    config: Option<PathBuf>,
    #[arg(short, long)]
    dictionary: Option<PathBuf>,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(
        long,
        help = "Validator: free-dictionary, merriam-webster, wordnik, custom"
    )]
    validator: Option<String>,
    #[arg(long, help = "API key for validators that require one")]
    api_key: Option<String>,
    #[arg(long, help = "Custom validator URL (use with --validator custom)")]
    validator_url: Option<String>,
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

    let mut config = if let Some(path) = args.config {
        match Config::from_file(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Config error: {}", e);
                process::exit(1);
            }
        }
    } else {
        Config::default()
    };

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

    // Parse validator from CLI flag
    let validator_kind = if let Some(v) = args.validator {
        match v.parse::<ValidatorKind>() {
            Ok(kind) => Some(kind),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        config.validator.clone()
    };

    let api_key = args.api_key.or(config.api_key.clone());
    let validator_url = args.validator_url.or(config.validator_url.clone());

    if config.letters.is_none() || config.present.is_none() {
        eprintln!("Error: letters and present letters are required.");
        process::exit(1);
    }

    let dictionary = match Dictionary::from_file(&config.dictionary) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Dictionary error: {}", e);
            eprintln!("Tip: Run 'make setup'.");
            process::exit(1);
        }
    };

    let solver = Solver::new(config.clone());

    match solver.solve(&dictionary) {
        Ok(words) => {
            let mut sorted_words: Vec<_> = words.into_iter().collect();
            sorted_words.sort();

            if let Some(kind) = validator_kind {
                let validator =
                    match create_validator(&kind, api_key.as_deref(), validator_url.as_deref()) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Validator error: {}", e);
                            process::exit(1);
                        }
                    };

                let mut entries = Vec::new();
                for word in &sorted_words {
                    match validator.lookup(word) {
                        Ok(Some(entry)) => entries.push(entry),
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("Warning: validation error for '{}': {}", word, e);
                        }
                    }
                }

                let output_fn = |entry: &sbs::WordEntry, f: &mut dyn Write| {
                    writeln!(f, "{}\t{}\t{}", entry.word, entry.definition, entry.url).unwrap();
                };

                if let Some(out_path) = config.output {
                    if let Ok(mut file) = File::create(out_path) {
                        for entry in &entries {
                            output_fn(entry, &mut file);
                        }
                    }
                } else {
                    for entry in &entries {
                        output_fn(entry, &mut std::io::stdout());
                    }
                }
            } else if let Some(out_path) = config.output {
                if let Ok(mut file) = File::create(out_path) {
                    for w in sorted_words {
                        writeln!(file, "{}", w).unwrap();
                    }
                }
            } else {
                for w in sorted_words {
                    println!("{}", w);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
