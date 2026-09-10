# haiku

A small Rust CLI for writing, checking, and collecting 5-7-5 haikus. Colors follow your Omarchy theme when available.

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

## Interactive mode

Run `haiku` with no arguments for a menu-driven session: pick an action by number, letter, or name (`1`/`n`/`new`, `2`/`c`/`check`, `3`/`l`/`list`, `4`/`r`/`random`, `5`/`i`/`import`, `6`/`q`/`quit`), and it loops back to the menu after each one until you quit.

## Import format

`import` expects a JSON array of strings, each holding 3 newline-separated lines, e.g.:

```json
["An old silent pond\nA frog jumps into the pond\nSplash! Silence again"]
```

(This is the shape used by e.g. [github.com/remy/haiku](https://github.com/remy/haiku)'s `db.json`.) Entries that don't parse as a valid 5-7-5 haiku are skipped rather than erroring, since real-world haiku collections mix in looser or mis-punctuated verse. Saving dedupes automatically, so re-importing the same file (or overlapping datasets) won't create duplicate entries.

Saved haikus live in the platform data directory (`~/.local/share/haiku/haikus.json` on Linux).

Syllable counts are a heuristic, not a linguist.
