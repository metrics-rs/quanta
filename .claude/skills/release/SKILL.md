---
name: release
description: >
  Run the release process for metrics-rs/quanta: audit unreleased changes,
  plan the version bump and changelog entries, then execute (cargo-release
  publish + per-PR comments + S-awaiting-release label removal) with explicit
  gates before anything is published. Invoke with /release when the user asks
  to prepare/run a release, says "what's ready to release?", or wants to drain
  the S-awaiting-release queue.
---

# Release

Patch/minor releases of the single `quanta` crate.

`cargo-release` is configured (`release.toml` at the repo root) to do the
`[Unreleased] -> [version] - date` substitution in `CHANGELOG.md`, update the
`[Unreleased]` compare link at the bottom of the file, and create signed
commits/tags. Tags follow the pattern `v<version>` (e.g. `v0.12.6`).

## Workflow at a glance

1. **Audit** — find unreleased changes since the last tag
2. **Plan** — version bump, draft changelog entries, list of PRs to notify
3. **Confirm the plan with the user** before any destructive action
4. **Execute** — changelog commit, dry-run, explicit user confirmation, then
   `cargo release --execute`, then PR comments + label removal
5. **Sanity check** — `S-awaiting-release` queue should be empty

## Phase 1 — Audit

Two complementary signals; the git log is authoritative because the label is
applied by hand and may be missing.

```bash
# Last released version / tag
grep '^version' Cargo.toml
git tag --sort=-creatordate | head -1

# Unreleased commits (authoritative)
git log --oneline v<current_version>..HEAD

# PRs explicitly flagged awaiting release (informative — may have gaps)
gh pr list --state merged --label "S-awaiting-release" --limit 100 \
  --json number,title,labels,mergedAt,url,author
```

Note: the `S-awaiting-release` label may not exist in this repository yet. If
the `gh pr list` call errors or returns nothing, rely entirely on the git log.

For each commit, extract the PR number (squash merges end in `(#NNN)`) and use
`git show --stat <sha>` / `gh pr view <NNN>` to understand what it changed and
who authored it. Commits pushed directly to `main` without a PR still get
changelog entries if user-facing, but have no PR to notify.

Also read the existing `## [Unreleased] - ReleaseDate` section in
`CHANGELOG.md` — entries are often added alongside the change itself, so some
(or all) of the changelog may already be written. Reconcile it against the git
log rather than duplicating entries.

Skip these from the audit:
- Pure `chore(deps): bump X from A to B` dependabot commits — they ride along
  in the release without dedicated changelog entries unless they have user-
  facing impact (e.g., MSRV bump, public dependency like `prost-types`).
- CI-only changes (`.github/` workflows, runner config) and other changes with
  no effect on the published crate.
- `<!-- next-header -->`-only changes from prior cargo-release runs.

## Phase 2 — Plan

### Version bump assessment

`quanta` is 0.x, so semver treats minor as breaking:

- **Patch** (`0.x.y` → `0.x.(y+1)`): all changes are additive (new methods,
  new trait impls, new platform support, `#[deprecated]` annotations,
  `const fn` qualifications, internal restructuring with private fields,
  performance changes, bug fixes).
- **Minor** (`0.x.y` → `0.(x+1).0`): any change that breaks downstream
  compilation — removed/renamed public items, changed signatures, changed
  trait bounds, public items moved behind a new (even default-enabled)
  feature flag, dropped target support, public dependency major bumps (e.g.
  `prost-types`).

Default to patch unless evidence of breakage exists. Verify by looking at
public API diffs (`pub` items, trait signatures, feature gates) — internal
changes (private field types, dependency swaps that don't leak into the API)
are not breaking even if behavior shifts. Entries in `CHANGELOG.md` explicitly
marked **Breaking change** are a strong signal for a minor bump.

MSRV bumps alone are generally treated as non-breaking for patch purposes
unless the user says otherwise, but call them out in the plan so the user can
decide.

### Changelog entries

Entries go under `## [Unreleased] - ReleaseDate` in `CHANGELOG.md`. Use
Keep-a-Changelog sub-headings: `### Added`, `### Changed`, `### Fixed`,
`### Deprecated`, `### Removed`. Bullets that correspond to a PR end with the
PR link: `([#NNN](https://github.com/metrics-rs/quanta/pull/NNN))`. Prefix
breaking changes with `**Breaking change:**`. Wrap lines at ~120 columns to
match the existing file.

Keep the literal `## [Unreleased] - ReleaseDate` header in place; cargo-release
substitutes it at release time per the `pre-release-replacements` in
`release.toml`. Likewise, leave the `<!-- next-header -->` marker and the
`<!-- next-url -->` / `[Unreleased]: ...compare/v<ver>...HEAD` link at the
bottom of the file untouched — cargo-release rewrites them and the
replacements use `exactly=1`, so it fails if they're missing or duplicated.

### PR comment scheduling

Every PR whose work ships in this release receives **exactly one comment**,
posted after `cargo release --execute` succeeds. The `S-awaiting-release`
label (where present) is removed in the same step.

#### Comment template

```
Released as `quanta@v<ver>`.

Thanks for your contribution! :heart:
```

#### Special cases

- **Maintainer's own PRs** (authored by `tobz`): skip the comment ("Thanks for
  your contribution!" on your own PR is awkward), but still remove the label.
- **PRs without `S-awaiting-release`**: still post a comment if their work
  shipped (it's fine to thank a contributor whose label was missing); just no
  label to remove.
- **Dependabot PRs**: no comment; remove the label if present.
- **CI-only / metadata-only PRs** (no effect on the published crate): comment
  is optional — skip by default unless the user disagrees.

## Phase 3 — Confirm with the user

Write the plan into a plan file (or surface it inline if no plan-mode file is
provided):

- Current → proposed version, with the reasoning for patch vs. minor
- List of PRs/commits by category (Added / Changed / Fixed / Deprecated /
  Removed), and the draft changelog diff (new entries + any edits to existing
  `Unreleased` entries)
- PR notification list: which PRs get a comment, which get label removal only,
  which are skipped and why

Wait for explicit user approval before any commits or `cargo release --execute`.

## Phase 4 — Execute

### a. Edit `CHANGELOG.md`

Insert/adjust the drafted bullets under `## [Unreleased] - ReleaseDate`. Keep
the literal `Unreleased` heading.

### b. Commit the changelog edit

cargo-release **requires a clean working tree**. Commit changelog edits in a
dedicated commit before invoking `cargo release` (skip this step if the
changelog needed no changes):

```bash
git add CHANGELOG.md
git commit -m "update CHANGELOG"
```

**GPG signing**: this repo uses signed commits (`commit.gpgsign = true`, and
`sign-commit`/`sign-tag` in `release.toml`). If the GPG agent isn't unlocked,
the commit fails with `gpg: signing failed: No such file or directory`
(pinentry can't reach a TTY from the agent context). Ask the user to unlock
GPG (`echo test | gpg --clearsign > /dev/null` from a TTY caches the
passphrase), then retry. **Do not bypass with `--no-gpg-sign`** unless the
user explicitly authorizes it.

### c. Dry-run cargo-release

```bash
cargo release patch     # or: cargo release minor
```

(No `--execute` flag = dry-run.) Inspect the output — confirm:
- Version bump is right (`Upgrading quanta from X.Y.Z to X.Y.(Z+1)`)
- CHANGELOG substitution looks correct (bullets land under `[X.Y.(Z+1)]`,
  fresh empty `[Unreleased]` added on top, compare link at the bottom updated)
- Tag/push target is `vX.Y.(Z+1)`

### d. Pause for user confirmation before `--execute`

Crates.io publishes are **permanent** (you can yank but not delete). Even with
auto mode active, this is a destructive/external action and needs explicit
user authorization. Show the dry-run summary and wait for "go ahead".

### e. Execute the release

```bash
cargo release patch --execute --no-confirm     # or: minor
```

This: bumps `Cargo.toml`/`Cargo.lock`, stamps `CHANGELOG.md`, creates a signed
`chore: Release quanta version X.Y.Z` commit + signed `vX.Y.Z` tag, publishes
to crates.io, pushes `main` + tag to origin.

### f. PR notifications

Post comments and remove labels per the schedule from Phase 2.

```bash
# Comment
gh pr comment <NUMBER> --repo metrics-rs/quanta --body "$(cat <<'EOF'
Released as `quanta@v<ver>`.

Thanks for your contribution! :heart:
EOF
)"
```

```bash
# Label removal — DO NOT use `gh pr edit --remove-label`. It currently fails
# with a GraphQL error about Projects (classic) deprecation:
#   GraphQL: Projects (classic) is being deprecated...
# Use the REST API instead:
gh api -X DELETE /repos/metrics-rs/quanta/issues/<NUMBER>/labels/S-awaiting-release
```

Comments and label removals on different PRs are independent — run them in
parallel within a single message of tool calls.

Surface a summary afterwards: released version, PR comments posted, labels
removed, anything skipped.

## Phase 5 — Sanity check

```bash
gh pr list --state merged --label "S-awaiting-release" --limit 100
```

Should be empty (modulo any PRs the user explicitly chose to skip).

Also useful:

```bash
git log --oneline -5                          # see the release commit
git tag --sort=-creatordate | head -3         # confirm the new tag
cargo search quanta --limit 1                 # confirm crates.io has the new version
```

## Reference: example invocation

```bash
# Phase 1 audit
git log --oneline v0.12.6..HEAD
gh pr list --state merged --label "S-awaiting-release" --limit 100 --json number,title,labels,url,author

# Phase 4
git add CHANGELOG.md && git commit -m "update CHANGELOG"
cargo release minor                                # dry-run, pause for user
cargo release minor --execute --no-confirm         # publishes after user "go"
gh pr comment 120 --repo metrics-rs/quanta --body "Released as \`quanta@v0.13.0\`.

Thanks for your contribution! :heart:"
gh api -X DELETE /repos/metrics-rs/quanta/issues/120/labels/S-awaiting-release
```

## Gotchas

- **GPG agent must be unlocked** before commits. Ask the user to unlock from a
  TTY-capable terminal first; don't bypass with `--no-gpg-sign`.
- **`gh pr edit --remove-label` is broken** (Projects-classic deprecation
  error). Use `gh api -X DELETE /repos/metrics-rs/quanta/issues/<n>/labels/S-awaiting-release`.
- **Deleting a label that isn't on the PR returns 404** — harmless, but only
  call the DELETE for PRs that actually carry the label.
- **Crates.io publish wait**: cargo-release may print
  "note: waiting for quanta v<ver> to be available at registry `crates-io`."
  This may take 30s–2m. Don't treat the wait as a hang.
- **Push warning about vulnerabilities**: pushes to main may print
  "GitHub found N vulnerabilities (...)". Unrelated to the release; mention it
  as a follow-up but don't block on it.
- **Mixed changelog link styles**: older entries use reference-style links
  (`[#95]: https://...` footnotes); newer entries use inline links. Use inline
  links for new entries.
