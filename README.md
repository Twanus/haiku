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

Linux x86_64 binary from the latest green `dev` build: [Releases](https://github.com/Twanus/haiku/releases/latest).

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
haiku                        # no args: full-screen TUI (browse & compose)
```

Saved haikus live in the platform data directory (`~/.local/share/haiku/haikus.json` on Linux). Writes are atomic and the store is locked; truncated JSON is refused rather than silently wiping the collection.

## TUI

Run `haiku` with no arguments for a full-screen interface with two screens, `Tab` to switch between them:

- **Browse** — type to substring-filter your saved haikus (case-insensitive, live), `↑`/`↓`/`PageUp`/`PageDown`/`Home`/`End` to move through matches, and a preview pane shows the selected haiku with its syllable counts. `Esc` clears the search first, then quits on an empty one.
- **Compose** — write a new haiku across three fields with the same live 5-7-5 feedback as `haiku new`. `Enter` advances to the next line once it hits its target syllable count; `Up`/`Shift+Tab` goes back to re-edit a previous line. `Ctrl+S` (or `Enter` on a completed third line) saves — it shows up in Browse immediately, no restart needed.
- `Ctrl+C` quits from either screen.

## Import format

`import` expects a JSON array of strings, each holding 3 newline-separated lines:

```json
["An old silent pond\nA frog jumps into the pond\nSplash! Silence again"]
```

That is the shape used by e.g. [github.com/remy/haiku](https://github.com/remy/haiku)'s `db.json`. Entries that don't parse as a valid 5-7-5 haiku are skipped rather than erroring, since real-world collections mix in looser or mis-punctuated verse. Saving dedupes automatically, so re-importing the same file (or overlapping datasets) will not create duplicate entries.

## CI

Land work on `dev`. `main` is ruleset-protected: direct pushes and PR merges are blocked. A green CI run on `dev` is the only way it moves.

Every push to `dev` or `main`, and every pull request, runs GitHub Actions:

| Job | What it does |
| --- | --- |
| **test** | `cargo test` — unit tests for syllable counting, 5-7-5 parsing, store persistence (atomic writes, locking, corrupt JSON), import, CLI helpers, and the TUI's state/key-handling (terminal-independent by design, so it's testable without a real TTY), plus end-to-end tests of the compiled binary (`check`, `new`, `list`, `random`, `import`). Then `cargo build --release`. |
| **audit** | `cargo audit` against the [RustSec](https://rustsec.org/) advisory database, so a known-vulnerable crate in `Cargo.lock` fails the build. Also runs weekly, even when dependencies have not changed. |
| **promote** | If both jobs are green on `dev`, fast-forwards `main` to that commit. |
| **release** | After promote, uploads a stripped Linux x86_64 binary to the [`latest` GitHub Release](https://github.com/Twanus/haiku/releases/latest). |

The badge at the top tracks `main`.

## License

[MIT](LICENSE)
