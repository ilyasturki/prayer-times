---
name: changelog
description: Draft a new version section in CHANGELOG.md for prayer-times by summarising commits since the last release tag. Use when the user says "update the changelog", "prep the changelog for release", "add changelog entry", or anything similar. Draft-only — does not run cargo release.
---

# Update CHANGELOG.md

This project's release flow is: edit `CHANGELOG.md`, then run `cargo release <level> --execute` (see `release.toml` and the README's Release process section). This skill owns the first half: producing the changelog entry.

## Format rules

- Follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) — the existing file already uses it. Do not invent new conventions.
- Section heading: `## [X.Y.Z] - YYYY-MM-DD`. Use today's date, resolved to an absolute `YYYY-MM-DD`.
- Subsection headings, in this order when present: `### Added`, `### Changed`, `### Deprecated`, `### Removed`, `### Fixed`, `### Security`. Omit any that have no entries.
- Bullets are short, user-facing, imperative-past ("Added timezone support", "Fixed Julian day calculation"). Group related commits into a single bullet rather than mirroring git history 1:1.
- Flag breaking changes explicitly with a leading `Breaking:` inside the `### Changed` or `### Removed` bullet — that's the convention used by the 0.4.0 entry.

## Steps

1. **Read the current CHANGELOG.md** to confirm format and find the most recent released version (the first `## [X.Y.Z]` section below `## [Unreleased]`, if any).
2. **Find the last release tag** with `git describe --tags --abbrev=0` (tags are `vX.Y.Z`). Fall back to the version detected in step 1 if the tag lookup fails.
3. **Survey commits since the last tag** with `git log <last-tag>..HEAD --no-merges --pretty=format:'%h %s'`. Read the commit bodies for any commit whose subject is unclear (`git show <sha>`).
4. **Ask the user for the target version** if it's ambiguous. Suggest one based on the commits: patch for fixes/chores, minor for new features, major for breaking changes. State your reasoning in one sentence and propose `X.Y.Z`.
5. **Draft the new section** in memory, grouping commits under the right Keep a Changelog subheadings. Drop purely internal noise (dependency bumps with no user impact, CI-only changes) unless the user has asked for a thorough log.
6. **Insert the section into CHANGELOG.md** immediately below `## [Unreleased]` and above the previous release's heading. Keep the `## [Unreleased]` header in place (empty) so the next cycle has somewhere to accumulate notes.
7. **Do not commit, do not bump `Cargo.toml`, do not run `cargo release`.** Those are the user's next step. End by telling the user the drafted version and suggesting the exact `cargo release` command (e.g. `cargo release 0.4.1 --execute` or `cargo release patch --execute`).

## What to skip

- Do not rewrite or reorder old released sections.
- Do not touch `Cargo.toml`'s `version` field — `cargo-release` owns that.
- Do not invent changes that aren't in the commit log. If the commits are too terse to summarise faithfully, say so and ask the user to clarify rather than making things up.
