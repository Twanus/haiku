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
        /// First line (5 syllables)
        line1: String,
        /// Second line (7 syllables)
        line2: String,
        /// Third line (5 syllables)
        line3: String,
        /// Validate but do not save
        #[arg(long)]
        dry_run: bool,
    },
    /// Check whether text is a 5-7-5 haiku
    Check {
        /// Haiku text. Reads stdin if omitted.
        text: Option<String>,
    },
    /// List saved haikus
    List,
    /// Print a random saved haiku
    Random,
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
            let haiku = Haiku::new(line1, line2, line3)?;
            if dry_run {
                println!("{haiku}");
            } else {
                store::save(&haiku)?;
                println!("saved:\n{haiku}");
            }
        }
        Command::Check { text } => {
            let body = match text {
                Some(text) => text,
                None => std::io::read_to_string(std::io::stdin())?,
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
