# Release manual

How to cut a versioned release of `haiku`. See [README.md](README.md#ci) for
what each CI job actually does — this is the step-by-step runbook for
*using* that pipeline.

## Before you start

- `dev` should already be green and promoted to `main` (check the CI badge,
  or `gh run list --branch dev --limit 1`). Don't bump the version on top
  of unvalidated work.
- Decide the new version number. This project doesn't promise strict
  semver API stability (it's a CLI/TUI, not a library), but the shape is
  worth following loosely:
  - **patch** (`x.y.Z`) — bug fixes, no behavior change a user would notice
    as a new capability.
  - **minor** (`x.Y.0`) — new commands, TUI features, anything additive.
  - **major** (`X.0.0`) — breaking changes to CLI flags/output, the TUI's
    keybindings, or the saved-haiku JSON store format.

## Cut the release

```bash
# 1. Bump the version in Cargo.toml, then sync Cargo.lock:
#    edit `version = "..."` under [package]
cargo build

# 2. Verify locally before pushing anything — matches what CI's lint/test
#    jobs actually run:
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings

# 3. Commit and push to dev:
git add Cargo.toml Cargo.lock
git commit -m "Bump version to X.Y.Z"
git push origin dev

# 4. Wait for ci.yml to go green and promote to main:
gh run list --branch dev --limit 1
# ...or watch the Actions tab. Don't proceed until `promote` has succeeded —
# the tag needs to point at a commit that's actually on main.

# 5. Tag that commit and push the tag — THIS is what triggers a release:
git tag -a vX.Y.Z -m "vX.Y.Z"
git push origin vX.Y.Z

# 6. Watch release.yml run (verify-version → build → publish):
gh run list --workflow=release.yml --limit 1
```

## Verify it landed

- The [Releases page](https://github.com/Twanus/haiku/releases) shows a new
  `vX.Y.Z` entry with both `haiku-x86_64-unknown-linux-gnu` and
  `haiku-x86_64-pc-windows-msvc.exe` attached.
- `https://github.com/Twanus/haiku/releases/latest` resolves to it
  (`gh api repos/Twanus/haiku/releases/latest --jq .tag_name`).

## If something goes wrong

- **`verify-version` fails** — the tag doesn't match `Cargo.toml`'s
  version. Delete the bad tag and retag once it's fixed:
  ```bash
  git tag -d vX.Y.Z
  git push origin :refs/tags/vX.Y.Z
  ```
- **`build` fails** (e.g. a Windows-only compile break) — this is the
  first point a release-profile, cross-platform build actually runs (see
  [README.md](README.md#ci)); `ci.yml`'s per-push checks don't catch this.
  Fix the issue on `dev`, let it promote, delete the bad tag as above, and
  retag.
- **Already published, but wrong** — delete the GitHub Release and its
  tag (`gh release delete vX.Y.Z`, then delete the tag as above), or just
  ship a new patch release rather than editing history other people may
  have already pulled.
