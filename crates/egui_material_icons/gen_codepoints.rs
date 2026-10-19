#!/usr/bin/env rust-script
//! This script generates the `src/icons.rs` file from the `MaterialSymbolsRounded-Regular.codepoints` file.
//! Install the `rust-script` crate with `cargo install rust-script` to run this script.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Suffixes of legacy names that, in Material Symbols, share the codepoint of
/// the name without them: the outline is a separate font, not a separate glyph.
const OUTLINE_SUFFIXES: [&str; 3] = ["_OUTLINE", "_OUTLINED", "_BORDER"];

pub fn main() {
    let codepoints = include_str!("./MaterialSymbolsRounded-Regular.codepoints");

    let entries: Vec<(String, &str)> = codepoints
        .split("\n")
        .map(str::trim)
        .filter_map(|point| {
            let split_point: Vec<&str> = point.split(" ").collect();
            (split_point.len() > 1).then(|| (split_point[0].to_uppercase(), split_point[1]))
        })
        .collect();
    // First occurrence wins, like the constants below.
    let mut by_name: HashMap<&str, &str> = HashMap::new();
    for (name, addr) in &entries {
        by_name.entry(name.as_str()).or_insert(addr);
    }

    let mut names = HashSet::new();

    let code: String = entries
        .iter()
        .filter_map(|(name, addr)| {
            if !names.insert(name.clone()) {
                return None;
            }
            // `ICON_DELETE_OUTLINE` would be `ICON_DELETE` under a name that
            // promises an outline. Skip it; use `ICON_DELETE.outlined()`.
            let outline_alias = OUTLINE_SUFFIXES.iter().any(|suffix| {
                name.strip_suffix(suffix)
                    .and_then(|base| by_name.get(base))
                    .is_some_and(|base_addr| base_addr == addr)
            });
            if outline_alias {
                return None;
            }
            Some(format!(
                "pub const ICON_{name}: crate::MaterialIcon = crate::MaterialIcon::new(\"\\u{{{addr}}}\");\n"
            ))
        })
        .collect();

    let file = PathBuf::try_from(file!()).unwrap();
    let icons = file.parent().unwrap().join("src").join("icons.rs");
    std::fs::write(icons, code).unwrap();

    println!("Generated src/icons.rs");
}
