use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::haiku::Haiku;
use crate::store;
use crate::syllables;

#[derive(Parser)]
#[command(
    name = "haiku",
    about = "Write, check, and collect haikus",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a haiku from three lines and save it
    New {
        /// First line (5 syllables). Prompts if omitted.
        #[arg(allow_hyphen_values = true)]
        line1: Option<String>,
        /// Second line (7 syllables). Prompts if omitted.
        #[arg(allow_hyphen_values = true)]
        line2: Option<String>,
        /// Third line (5 syllables). Prompts if omitted.
        #[arg(allow_hyphen_values = true)]
        line3: Option<String>,
        /// Validate but do not save
        #[arg(long)]
        dry_run: bool,
    },
    /// Check whether text is a 5-7-5 haiku
    Check {
        /// Haiku text. Reads stdin if omitted and --file is not set.
        #[arg(conflicts_with = "file", allow_hyphen_values = true)]
        text: Option<String>,
        /// Read haiku text from a file
        #[arg(long, value_name = "PATH")]
        file: Option<PathBuf>,
    },
    /// List saved haikus
    List,
    /// Print a random saved haiku
    Random,
}

fn read_line_prompt(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

/// Syllable count and per-word breakdown for one candidate haiku line.
struct LineCheck {
    count: u32,
    target: u32,
    breakdown: String,
}

impl LineCheck {
    fn is_ok(&self) -> bool {
        self.count == self.target
    }
}

fn check_line(line: &str, target: u32) -> LineCheck {
    let mut count = 0u32;
    let mut parts = Vec::new();
    for word in line.split_whitespace() {
        let n = syllables::count(word);
        count += n;
        parts.push(format!("{word}({n})"));
    }
    LineCheck {
        count,
        target,
        breakdown: parts.join(" "),
    }
}

/// Read one line (or use `initial`, if given), report its syllable count,
/// and keep re-prompting until it hits `target` syllables.
fn resolve_line(
    number: usize,
    target: u32,
    initial: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut candidate = initial;
    loop {
        let line = match candidate.take() {
            Some(line) => line,
            None => read_line_prompt(&format!("{number}> "))?,
        };
        let trimmed = line.trim();
        let check = check_line(trimmed, target);

        if check.is_ok() {
            println!(
                "  {}/{} syllables: {}",
                check.count, check.target, check.breakdown
            );
            return Ok(trimmed.to_string());
        }

        if trimmed.is_empty() {
            println!("  line {number} needs {target} syllables, got none — try again:");
        } else {
            println!(
                "  {}/{} syllables (need {}): {}",
                check.count, check.target, check.target, check.breakdown
            );
            println!("  try again:");
        }
    }
}

fn resolve_new_lines(
    line1: Option<String>,
    line2: Option<String>,
    line3: Option<String>,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    if line1.is_none() && line2.is_none() && line3.is_none() {
        println!("Enter three lines (5-7-5). I'll tell you the syllable count and let you fix any line that's off.");
    }

    let line1 = resolve_line(1, 5, line1)?;
    let line2 = resolve_line(2, 7, line2)?;
    let line3 = resolve_line(3, 5, line3)?;
    Ok((line1, line2, line3))
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::New {
            line1,
            line2,
            line3,
            dry_run,
        } => {
            let (line1, line2, line3) = resolve_new_lines(line1, line2, line3)?;
            let haiku = Haiku::new(line1, line2, line3)?;
            if dry_run {
                println!("{haiku}");
            } else {
                let rendered = haiku.to_string();
                store::save(haiku)?;
                println!("saved:\n{rendered}");
            }
        }
        Command::Check { text, file } => {
            let body = if let Some(path) = file {
                std::fs::read_to_string(&path).map_err(|err| {
                    format!("couldn't read {}: {err}", path.display())
                })?
            } else if let Some(text) = text {
                text
            } else {
                std::io::read_to_string(std::io::stdin())?
            };
            let haiku = Haiku::parse(&body)?;
            let [a, b, c] = haiku.syllable_counts();
            println!("{haiku}");
            println!("{a}-{b}-{c} ok");
        }
        Command::List => {
            let all = store::list()?;
            if all.is_empty() {
                println!("no haikus saved yet");
            } else {
                for (i, haiku) in all.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    let [a, b, c] = haiku.syllable_counts();
                    println!("{}. ({a}-{b}-{c})", i + 1);
                    println!("{haiku}");
                }
            }
        }
        Command::Random => {
            let haiku = store::random()?;
            println!("{haiku}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_line_reports_matching_count_and_breakdown() {
        let check = check_line("an old silent pond", 5);
        assert!(check.is_ok());
        assert_eq!(check.count, 5);
        assert_eq!(check.breakdown, "an(1) old(1) silent(2) pond(1)");
    }

    #[test]
    fn check_line_flags_too_few_syllables() {
        let check = check_line("an old silent", 5);
        assert!(!check.is_ok());
        assert_eq!(check.count, 4);
    }

    #[test]
    fn check_line_flags_too_many_syllables() {
        let check = check_line("a frog jumps into the pond and splashes", 7);
        assert!(!check.is_ok());
        assert!(check.count > 7);
    }

    #[test]
    fn check_line_empty_line_is_not_ok() {
        let check = check_line("", 5);
        assert!(!check.is_ok());
        assert_eq!(check.count, 0);
        assert_eq!(check.breakdown, "");
    }
}
