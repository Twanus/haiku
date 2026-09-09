# haiku

A small Rust CLI for writing, checking, and collecting 5-7-5 haikus.

```bash
haiku new "old pond" "a frog jumps in" "the sound of water"
haiku check
haiku list
haiku random
```

Saved haikus live in the platform data directory (`~/.local/share/haiku/haikus.json` on Linux).

Syllable counts are a heuristic, not a linguist.
