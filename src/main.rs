mod cli;
mod haiku;
mod store;
mod style;
mod syllables;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{}", style::error(&err.to_string()));
        std::process::exit(1);
    }
}
