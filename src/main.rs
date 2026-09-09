mod cli;
mod haiku;
mod store;
mod syllables;

fn main() {
    if let Err(err) = cli::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
