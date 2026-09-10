mod cli;
mod haiku;
mod import;
mod line_check;
mod store;
mod style;
mod syllables;
mod tui;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{}", style::error(&err.to_string()));
        std::process::exit(1);
    }
}
