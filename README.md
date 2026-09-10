# haiku

[![CI](https://github.com/Twanus/haiku/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Twanus/haiku/actions/workflows/ci.yml?query=branch%3Amain)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

```
an old silent pond
a frog jumps into the pond
splash silence again
```

A small, fast Rust CLI for writing, checking, and collecting 5-7-5 haikus — a plain command for scripting, and a full-screen TUI for browsing and composing. Colors follow your [Omarchy](https://omarchy.org) theme when available, falling back to the terminal palette otherwise.

Syllable counts are a heuristic, not a linguist — good enough to catch the obvious misses, not a substitute for reading your own haiku out loud.

## Contents

- [Install](#install)
- [Usage](#usage)
- [TUI](#tui)
- [Import](#import)
- [Project layout](#project-layout)
- [CI](#ci)
- [License](#license)

## Install

Prebuilt Linux and Windows x86_64 binaries for every tagged version: **[Releases](https://github.com/Twanus/haiku/releases/latest)**.

Or build it yourself:

```bash
git clone https://github.com/Twanus/haiku.git
cd haiku
cargo install --path .
```

Or run it straight from the repo without installing:

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

Saved haikus live in the platform data directory (`~/.local/share/haiku/haikus.json` on Linux). Writes are atomic and the store is locked while writing; truncated or corrupt JSON is refused with an error rather than silently wiping the collection.

## TUI

Run `haiku` with no arguments for a full-screen interface with two screens — `Tab` to switch between them, `Ctrl+C` to quit from either:

```
┌ search ────────────────────────────────────────────────────────────────┐
│› pond                                                                  │
└────────────────────────────────────────────────────────────────────────┘
┌ browse (2/1247) ───────────────────────┐┌ preview ─────────────────────┐
│❯ an old silent pond / a frog...        ││an old silent pond            │
│  wind through reeds / a heron...       ││· a frog jumps into the pond  │
│                                        ││splash silence again          │
│                                        ││                              │
│                                        ││5-7-5                         │
└────────────────────────────────────────┘└──────────────────────────────┘
 type to search · ↑↓ move · Tab compose · Esc back/quit · Ctrl+C quit
```

- **Browse** — type to substring-filter your saved haikus (case-insensitive, live), `↑`/`↓`/`PageUp`/`PageDown`/`Home`/`End` to move through matches, and a preview pane shows the selected haiku with its syllable counts. `Esc` clears the search first, then quits on an empty one.
- **Compose** — write a new haiku across three fields with the same live 5-7-5 feedback as `haiku new`. `Enter` advances to the next line once it hits its target syllable count; `Up` / `Shift+Tab` goes back to re-edit a previous line. `Ctrl+S` (or `Enter` on a completed third line) saves — it shows up in Browse immediately, no restart needed.

Both search and save stay fast no matter how large the store gets: search narrows within the current results instead of rescanning everything on every keystroke, and only the currently visible rows ever get rendered.

## Import

`import` expects a JSON array of strings, each holding 3 newline-separated lines:

```json
["An old silent pond\nA frog jumps into the pond\nSplash! Silence again"]
```

That's the shape used by e.g. [github.com/remy/haiku](https://github.com/remy/haiku)'s `db.json`. Entries that don't parse as a valid 5-7-5 haiku are skipped rather than erroring, since real-world collections mix in looser or mis-punctuated verse. Saving dedupes automatically, so re-importing the same file (or overlapping datasets) never creates duplicate entries.

## Project layout

```
src/
├── main.rs   entry point
├── cli.rs    CLI frontend — argument parsing, subcommands, line prompts
├── tui/      TUI frontend — terminal lifecycle, state, rendering
├── domain/   pure haiku logic, no I/O — model, syllable counting, import parsing
└── infra/    system boundaries — the saved-haiku store, terminal theme loading
```

Both frontends are built on the same `domain` rules and the same `infra` — nothing haiku-domain-specific talks to a file or a terminal directly outside `infra`, which is what keeps `domain` (and most of `tui`) unit-testable without touching disk or a real TTY.

## CI

Land work on `dev`. `main` is ruleset-protected: direct pushes and PR merges are blocked, so a green CI run on `dev` is the only way it moves.

Every push to `dev` or `main`, and every pull request, runs `ci.yml`:

| Job | What it does |
| --- | --- |
| **test** | `cargo test --locked` — unit tests for syllable counting, 5-7-5 parsing, store persistence (atomic writes, locking, corrupt JSON), import, CLI helpers, and the TUI's state/key-handling (terminal-independent by design, so it's testable without a real TTY), plus end-to-end tests of the compiled binary (`check`, `new`, `list`, `random`, `import`). |
| **audit** | `cargo audit` against the [RustSec](https://rustsec.org/) advisory database, so a known-vulnerable crate in `Cargo.lock` fails the build. Also runs weekly, even when dependencies haven't changed. |
| **lint** | `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`. |
| **promote** | If test/audit/lint are all green on `dev`, fast-forwards `main` to that commit. |

Cutting an actual release is a separate, deliberate step, not something every push does: bump `version` in `Cargo.toml`, let it promote to `main` as normal, then push a matching tag (`git tag -a v1.1.0 -m "..." && git push origin v1.1.0`). That triggers `release.yml`:

| Job | What it does |
| --- | --- |
| **verify-version** | Fails fast if the pushed tag doesn't match `Cargo.toml`'s version — catches a forgotten bump or a typo'd tag before any build runs. |
| **build** | Builds and packages the Linux and Windows x86_64 release binaries. |
| **publish** | Publishes them to a GitHub Release named after the tag, marked as `latest` — every past version stays listed on the [Releases page](https://github.com/Twanus/haiku/releases). |

See [release-manual.md](release-manual.md) for the full step-by-step runbook, including what to do if a release fails partway through.

The badge at the top tracks `main`.

## License

[MIT](LICENSE)
