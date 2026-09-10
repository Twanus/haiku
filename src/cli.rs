use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::haiku::Haiku;
use crate::import;
use crate::store;
use crate::style;
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
    /// Import haikus from a JSON array of raw haiku text
    Import {
        /// Path to a JSON file: an array of strings, each 3 newline-separated lines
        path: PathBuf,
        /// Report what would be imported without saving anything
        #[arg(long)]
        dry_run: bool,
    },
}

/// Read one line, pre-filling `initial` as editable text when stdin is a
/// real terminal (so a rejected line can be corrected instead of retyped).
/// Falls back to a plain read when stdin isn't interactive (e.g. piped),
/// where line editing isn't possible — an empty line there just repeats
/// `initial` as-is.
fn read_line_prompt(prompt: &str, initial: &str) -> Result<String, Box<dyn std::error::Error>> {
    if io::stdin().is_terminal() {
        let mut editor = rustyline::DefaultEditor::new()?;
        let line = editor.readline_with_initial(prompt, (initial, ""))?;
        return Ok(line.trim().to_string());
    }

    print!("{prompt}");
    io::stdout().flush()?;
    let mut buf = String::new();
    let bytes_read = io::stdin().read_line(&mut buf)?;
    if bytes_read == 0 {
        return Err("unexpected end of input while waiting for a line".into());
    }
    let buf = buf.trim();
    Ok(if buf.is_empty() {
        initial.to_string()
    } else {
        buf.to_string()
    })
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

fn format_ok_feedback(check: &LineCheck) -> String {
    format!(
        "  {} {}/{} syllables: {}",
        style::success(style::OK),
        style::success(&check.count.to_string()),
        check.target,
        style::muted(&check.breakdown)
    )
}

fn format_bad_feedback(check: &LineCheck, number: usize) -> String {
    if check.breakdown.is_empty() {
        format!(
            "  {} line {number} needs {} syllables, got none {}",
            style::error(style::BAD),
            style::warn(&check.target.to_string()),
            style::muted("— try again:")
        )
    } else {
        format!(
            "  {} {}/{} syllables (need {}): {}\n  {}",
            style::error(style::BAD),
            style::warn(&check.count.to_string()),
            check.target,
            check.target,
            style::muted(&check.breakdown),
            style::muted("try again (edit the line below):")
        )
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
    let mut prefill = String::new();
    let prompt = format!(
        "{}{} ",
        style::accent(&format!("{number}")),
        style::accent(style::PROMPT)
    );
    loop {
        let line = match candidate.take() {
            Some(line) => line,
            None => read_line_prompt(&prompt, &prefill)?,
        };
        let trimmed = line.trim();
        let check = check_line(trimmed, target);

        if check.is_ok() {
            println!("{}", format_ok_feedback(&check));
            return Ok(trimmed.to_string());
        }

        prefill = trimmed.to_string();
        println!("{}", format_bad_feedback(&check, number));
    }
}

fn resolve_new_lines(
    line1: Option<String>,
    line2: Option<String>,
    line3: Option<String>,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    if line1.is_none() && line2.is_none() && line3.is_none() {
        println!(
            "{} Enter three lines (5-7-5). I'll show syllable counts and let you fix any line that's off.",
            style::accent(style::FLOWER)
        );
    }

    let line1 = resolve_line(1, 5, line1)?;
    let line2 = resolve_line(2, 7, line2)?;
    let line3 = resolve_line(3, 5, line3)?;
    Ok((line1, line2, line3))
}

fn print_haiku_block(haiku: &Haiku) {
    for (i, line) in haiku.lines.iter().enumerate() {
        let mark = if i == 1 {
            style::accent(style::DOT)
        } else {
            style::muted(style::DOT)
        };
        println!("  {mark} {}", style::fg(line));
    }
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
                println!(
                    "{} {}",
                    style::success(style::OK),
                    style::muted("looks good")
                );
                print_haiku_block(&haiku);
            } else {
                let rendered = haiku.clone();
                store::save(haiku)?;
                println!(
                    "{} {}",
                    style::success(style::OK),
                    style::success("saved")
                );
                print_haiku_block(&rendered);
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
            print_haiku_block(&haiku);
            println!(
                "{} {}-{}-{} ok",
                style::success(style::OK),
                style::success(&a.to_string()),
                style::success(&b.to_string()),
                style::success(&c.to_string())
            );
        }
        Command::List => {
            let all = store::list()?;
            if all.is_empty() {
                println!(
                    "{} {}",
                    style::muted(style::FLOWER),
                    style::muted("no haikus saved yet")
                );
            } else {
                for (i, haiku) in all.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    let [a, b, c] = haiku.syllable_counts();
                    println!(
                        "{} {} {}",
                        style::accent(&format!("{}.", i + 1)),
                        style::cyan(&format!("({a}-{b}-{c})")),
                        style::muted(style::FLOWER)
                    );
                    print_haiku_block(haiku);
                }
            }
        }
        Command::Random => {
            let haiku = store::random()?;
            println!(
                "{} {}",
                style::accent(style::FLOWER),
                style::muted("a random haiku")
            );
            print_haiku_block(&haiku);
        }
        Command::Import { path, dry_run } => {
            let body = std::fs::read_to_string(&path)
                .map_err(|err| format!("couldn't read {}: {err}", path.display()))?;
            let report = import::parse(&body)?;
            let count = report.imported.len();
            if !dry_run {
                for haiku in &report.imported {
                    store::save(haiku.clone())?;
                }
            }
            println!(
                "{} imported {}, skipped {}",
                style::success(style::OK),
                style::success(&count.to_string()),
                style::warn(&report.skipped.len().to_string())
            );
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
