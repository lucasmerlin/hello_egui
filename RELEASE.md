How to release:

- Update all changelogs
- Run `cargo run --bin update_badges -p scripts` to update badges
- Run either
    - `cargo release minor --workspace` to release a new minor for all crates (useful on egui updates)
        - note that you shouldn't use this to release a new crate (since it'll be updated from 0.1.0 to 0.2.0)
    - `cargo release -p <crate_name> <patch|minor|major>` to release a new version for a single crate
- Confirm that all listed crates have a updated changelog with matching versions
- Run the release command with --execute
