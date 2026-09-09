# haiku

A small Rust CLI for writing, checking, and collecting 5-7-5 haikus.

```bash
haiku new "an old silent pond" "a frog jumps into the pond" "splash silence again"
haiku new                    # prompts 1> 2> 3>
haiku check --file poem.txt
haiku list
haiku random
```

Saved haikus live in the platform data directory (`~/.local/share/haiku/haikus.json` on Linux).

Syllable counts are a heuristic, not a linguist.
