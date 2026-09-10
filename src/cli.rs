use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::haiku::Haiku;
use crate::import;
use crate::line_check::{check_line, LineCheck};
use crate::store;
use crate::style;

#[derive(Parser)]
#[command(
    name = "haiku",
    about = "Write, check, and collect haikus",
    version
)]
struct Cli {
    /// Launch the TUI if no subcommand is given.
    #[command(subcommand)]
    command: Option<Command>,
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

fn cmd_new(
    line1: Option<String>,
    line2: Option<String>,
    line3: Option<String>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

fn cmd_check(text: Option<String>, file: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let body = if let Some(path) = file {
        std::fs::read_to_string(&path)
            .map_err(|err| format!("couldn't read {}: {err}", path.display()))?
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
    Ok(())
}

fn cmd_list() -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

fn cmd_random() -> Result<(), Box<dyn std::error::Error>> {
    let haiku = store::random()?;
    println!(
        "{} {}",
        style::accent(style::FLOWER),
        style::muted("a random haiku")
    );
    print_haiku_block(&haiku);
    Ok(())
}

fn cmd_import(path: PathBuf, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let body = std::fs::read_to_string(&path)
        .map_err(|err| format!("couldn't read {}: {err}", path.display()))?;
    let report = import::parse(&body)?;
    let count = report.imported.len();
    let skipped = report.skipped.len();
    if !dry_run && count > 0 {
        store::save_all(report.imported)?;
    }
    println!(
        "{} imported {}, skipped {}",
        style::success(style::OK),
        style::success(&count.to_string()),
        style::warn(&skipped.to_string())
    );
    Ok(())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::New {
            line1,
            line2,
            line3,
            dry_run,
        }) => cmd_new(line1, line2, line3, dry_run)?,
        Some(Command::Check { text, file }) => cmd_check(text, file)?,
        Some(Command::List) => cmd_list()?,
        Some(Command::Random) => cmd_random()?,
        Some(Command::Import { path, dry_run }) => cmd_import(path, dry_run)?,
        None => crate::tui::run()?,
    }
    Ok(())
}
