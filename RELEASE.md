# How to release

Releases are automated with [release-plz](https://release-plz.dev).
The config lives in `release-plz.toml`, the workflow in
`.github/workflows/release-plz.yml`.

- Every push to `main` updates a release PR that bumps the versions and
  changelogs of all crates that changed since their last release.
- Merging the release PR publishes the crates to crates.io (via trusted
  publishing), tags each crate (`<crate>-v<version>`), and creates a GitHub
  release for `hello_egui` whose notes list the changes of all crates.
- Nothing is published on other pushes (`release_always = false`).

## Changelogs

Changelog entries are generated from the commit messages on `main` — usually
the squash-merged PR titles. Write PR titles with the changelog in mind.

You can edit the release PR before merging it, to reword changelog entries or
to change versions (manually or with `release-plz set-version
<crate>@<version>`, which also updates the pending changelog section).

## Version bumps

release-plz proposes a patch bump for each changed crate, except when:

- the version on `main` is already higher than the published one — then it
  keeps that version,
- cargo-semver-checks detects an API breaking change — then minor (for 0.x),
- egui was updated — then the workflow raises the bump to minor (see below).

## egui updates

An egui update is a breaking change for every crate, but the commit typically
only touches the workspace `Cargo.toml`, so release-plz cannot attribute it to
the individual crates and would propose patch bumps.

The workflow handles this automatically: when the egui version on `main` is
newer than the one the crates were last published against (checked via the
crates.io index), CI runs `scripts/bump_versions_on_egui_update.sh` on the
release PR branch. The script raises every pending patch bump to the next
minor version and puts an "Update egui to 0.x" entry into each crate's
changelog section. CI pushes the fix-up to the release PR as a bot commit, so
release-plz can still regenerate the PR when `main` moves (the fix-up is then
re-applied by the next run).

The script is idempotent (crates whose pending version already bumps the minor
are left alone) and can also be run locally on a checked-out release PR
branch.

Install release-plz with `brew install release-plz` or
`cargo binstall release-plz`.

## New crates

crates.io trusted publishing does not cover the first publish of a crate:

1. Publish it once manually: `cargo publish -p <crate>`.
2. On crates.io, add the trusted publishing config (Settings → Trusted
   Publishing): repository `lucasmerlin/hello_egui`, workflow
   `release-plz.yml`.

## Badges

Run `cargo run -p scripts --bin update_badges` to update the README badges
after adding a crate.
