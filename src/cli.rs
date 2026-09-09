use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::haiku::Haiku;
use crate::store;

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
        line1: Option<String>,
        /// Second line (7 syllables). Prompts if omitted.
        line2: Option<String>,
        /// Third line (5 syllables). Prompts if omitted.
        line3: Option<String>,
        /// Validate but do not save
        #[arg(long)]
        dry_run: bool,
    },
    /// Check whether text is a 5-7-5 haiku
    Check {
        /// Haiku text. Reads stdin if omitted and --file is not set.
        #[arg(conflicts_with = "file")]
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

fn resolve_new_lines(
    line1: Option<String>,
    line2: Option<String>,
    line3: Option<String>,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    let interactive = line1.is_none() && line2.is_none() && line3.is_none();
    if interactive {
        println!("Enter three lines (5-7-5):");
        let line1 = read_line_prompt("1> ")?;
        let line2 = read_line_prompt("2> ")?;
        let line3 = read_line_prompt("3> ")?;
        return Ok((line1, line2, line3));
    }

    match (line1, line2, line3) {
        (Some(a), Some(b), Some(c)) => Ok((a, b, c)),
        _ => Err(
            "provide all three lines, or omit all three to enter them interactively".into(),
        ),
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
                println!("{haiku}");
            } else {
                let rendered = haiku.to_string();
                store::save(haiku)?;
                println!("saved:\n{rendered}");
            }
        }
        Command::Check { text, file } => {
            let body = if let Some(path) = file {
                std::fs::read_to_string(path)?
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
