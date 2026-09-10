mod cli;
mod domain;
mod infra;
mod tui;

use infra::style;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{}", style::error(&err.to_string()));
        std::process::exit(1);
    }
}
