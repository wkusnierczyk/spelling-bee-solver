//! CLI entry point for Spelling Bee Solver.

use clap::Parser;
#[cfg(feature = "validator")]
use sbs::{create_validator, ValidatorKind};
use sbs::{Config, Dictionary, Solver};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "sbs")]
#[command(version)]
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
    #[cfg(feature = "validator")]
    #[arg(
        long,
        help = "Validator: free-dictionary, merriam-webster, wordnik, custom"
    )]
    validator: Option<String>,
    #[cfg(feature = "validator")]
    #[arg(long, help = "API key for validators that require one")]
    api_key: Option<String>,
    #[cfg(feature = "validator")]
    #[arg(long, help = "Custom validator URL (use with --validator custom)")]
    validator_url: Option<String>,
    #[arg(long)]
    minimal_word_length: Option<usize>,
    #[arg(long)]
    maximal_word_length: Option<usize>,
    #[arg(
        long,
        default_value = "plain",
        help = "Output format: plain, json, markdown"
    )]
    format: String,
    #[arg(long)]
    case_sensitive: bool,
    #[arg(long)]
    about: bool,
}

fn print_about() {
    println!("sbs: Spelling Bee Solver tool");
    println!("├─ version:   {}", env!("CARGO_PKG_VERSION"));
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
    if let Some(n) = args.minimal_word_length {
        config.minimal_word_length = Some(n);
    }
    if let Some(n) = args.maximal_word_length {
        config.maximal_word_length = Some(n);
    }
    if args.case_sensitive {
        config.case_sensitive = Some(true);
    }

    // Parse validator from CLI flag
    #[cfg(feature = "validator")]
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

    #[cfg(feature = "validator")]
    let api_key = args.api_key.or(config.api_key.clone());
    #[cfg(feature = "validator")]
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

    let format = args.format.as_str();
    if !matches!(format, "plain" | "json" | "markdown") {
        eprintln!(
            "Error: unsupported format '{}'. Use plain, json, or markdown.",
            format
        );
        process::exit(1);
    }

    match solver.solve(&dictionary) {
        Ok(words) => {
            let mut sorted_words: Vec<_> = words.into_iter().collect();
            sorted_words.sort();

            #[cfg(feature = "validator")]
            let validated = if let Some(kind) = validator_kind {
                let validator =
                    match create_validator(&kind, api_key.as_deref(), validator_url.as_deref()) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Validator error: {}", e);
                            process::exit(1);
                        }
                    };

                let summary = validator.validate_words(&sorted_words);
                eprintln!(
                    "Generated {} candidates, {} validated by {}.",
                    summary.candidates,
                    summary.validated,
                    kind.display_name()
                );

                let output = format_validated(&summary.entries, format);
                write_output(&output, config.output.as_deref());
                true
            } else {
                false
            };

            #[cfg(feature = "validator")]
            if validated {
                return;
            }

            eprintln!("Generated {} words.", sorted_words.len());

            let output = format_unvalidated(&sorted_words, format);
            write_output(&output, config.output.as_deref());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn format_unvalidated(words: &[String], format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(words).unwrap(),
        "markdown" => words
            .iter()
            .map(|w| format!("**{}**", w))
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => words.join("\n"),
    }
}

#[cfg(feature = "validator")]
fn format_validated(entries: &[sbs::WordEntry], format: &str) -> String {
    match format {
        "json" => serde_json::to_string_pretty(entries).unwrap(),
        "markdown" => entries
            .iter()
            .map(|e| format!("**{}**\n{}", e.word, e.definition))
            .collect::<Vec<_>>()
            .join("\n\n"),
        _ => entries
            .iter()
            .map(|e| format!("{}\t{}", e.word, e.definition))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn write_output(content: &str, out_path: Option<&str>) {
    if let Some(path) = out_path {
        match File::create(path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(content.as_bytes()) {
                    eprintln!("Write error: {}", e);
                    process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to create output file '{}': {}", path, e);
                process::exit(1);
            }
        }
    } else {
        println!("{}", content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_unvalidated_plain() {
        let words = vec!["apple".to_string(), "bat".to_string()];
        assert_eq!(format_unvalidated(&words, "plain"), "apple\nbat");
    }

    #[test]
    fn test_format_unvalidated_json() {
        let words = vec!["apple".to_string(), "bat".to_string()];
        let output = format_unvalidated(&words, "json");
        let parsed: Vec<String> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed, vec!["apple", "bat"]);
    }

    #[test]
    fn test_format_unvalidated_markdown() {
        let words = vec!["apple".to_string(), "bat".to_string()];
        assert_eq!(
            format_unvalidated(&words, "markdown"),
            "**apple**\n\n**bat**"
        );
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_format_validated_plain() {
        let entries = vec![sbs::WordEntry {
            word: "apple".to_string(),
            definition: "A fruit".to_string(),
            url: "https://example.com/apple".to_string(),
        }];
        assert_eq!(format_validated(&entries, "plain"), "apple\tA fruit");
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_format_validated_json() {
        let entries = vec![sbs::WordEntry {
            word: "apple".to_string(),
            definition: "A fruit".to_string(),
            url: "https://example.com/apple".to_string(),
        }];
        let output = format_validated(&entries, "json");
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed[0]["word"], "apple");
        assert_eq!(parsed[0]["definition"], "A fruit");
    }

    #[cfg(feature = "validator")]
    #[test]
    fn test_format_validated_markdown() {
        let entries = vec![sbs::WordEntry {
            word: "apple".to_string(),
            definition: "A fruit".to_string(),
            url: "https://example.com/apple".to_string(),
        }];
        assert_eq!(format_validated(&entries, "markdown"), "**apple**\nA fruit");
    }
}
