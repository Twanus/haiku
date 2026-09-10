# haiku

[![CI](https://github.com/Twanus/haiku/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Twanus/haiku/actions/workflows/ci.yml?query=branch%3Amain)

```
an old silent pond
a frog jumps into the pond
splash silence again
```

A small Rust CLI for writing, checking, and collecting 5-7-5 haikus. Colors follow your Omarchy theme when available; otherwise they fall back to the terminal palette.

Syllable counts are a heuristic, not a linguist.

## Install

```bash
git clone https://github.com/Twanus/haiku.git
cd haiku
cargo install --path .
```

Or run from the repo without installing:

```bash
cargo run --release -- new "an old silent pond" "a frog jumps into the pond" "splash silence again"
```

## Usage

```bash
haiku new "an old silent pond" "a frog jumps into the pond" "splash silence again"
haiku new                    # prompts 1> 2> 3>, with live syllable feedback and per-line retry
haiku new --dry-run ...      # validate without saving
haiku check --file poem.txt
haiku check < poem.txt
haiku list
haiku random
haiku import haikus.json     # bulk-load from a JSON array of raw haiku text
haiku import haikus.json --dry-run
haiku                        # no args: interactive menu for all of the above
```

Saved haikus live in the platform data directory (`~/.local/share/haiku/haikus.json` on Linux). Writes are atomic and the store is locked; truncated JSON is refused rather than silently wiping the collection.

## Interactive mode

Run `haiku` with no arguments for a menu-driven session. Pick an action by number, letter, or name (`1`/`n`/`new`, `2`/`c`/`check`, `3`/`l`/`list`, `4`/`r`/`random`, `5`/`i`/`import`, `6`/`q`/`quit`). It loops back to the menu after each one until you quit.

## Import format

`import` expects a JSON array of strings, each holding 3 newline-separated lines:

```json
["An old silent pond\nA frog jumps into the pond\nSplash! Silence again"]
```

That is the shape used by e.g. [github.com/remy/haiku](https://github.com/remy/haiku)'s `db.json`. Entries that don't parse as a valid 5-7-5 haiku are skipped rather than erroring, since real-world collections mix in looser or mis-punctuated verse. Saving dedupes automatically, so re-importing the same file (or overlapping datasets) will not create duplicate entries.

## CI

Every push to `main` or `dev`, and every pull request, runs GitHub Actions:

| Job | What it does |
| --- | --- |
| **test** | `cargo test` — unit tests for syllable counting, 5-7-5 parsing, store persistence (atomic writes, locking, corrupt JSON), import, and CLI helpers, plus end-to-end tests of the compiled binary (`check`, `new`, `list`, `random`, `import`). Then `cargo build --release`. |
| **audit** | `cargo audit` against the [RustSec](https://rustsec.org/) advisory database, so a known-vulnerable crate in `Cargo.lock` fails the build. Also runs weekly, even when dependencies have not changed. |
| **promote** | If both jobs are green on `dev`, fast-forwards `main` to that commit. |

The badge at the top tracks `main`.
