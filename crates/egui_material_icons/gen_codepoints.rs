#!/usr/bin/env rust-script
//! This script generates the `src/icons.rs` file from the `MaterialSymbolsRounded-Regular.codepoints` file.
//! Install the `rust-script` crate with `cargo install rust-script` to run this script.

use std::collections::HashSet;
use std::path::PathBuf;

pub fn main() {
    let codepoints = include_str!("./MaterialSymbolsRounded-Regular.codepoints");

    let mut names = HashSet::new();

    let code: String = codepoints
        .split("\n")
        .map(str::trim)
        .filter_map(|point| {
            let split_point: Vec<&str> = point.split(" ").collect();

            if split_point.len() > 1 {
                let name = split_point[0].to_uppercase();
                let addr = split_point[1];

                if !names.contains(&name) {
                    let token = Some(format!(
                        "pub const ICON_{name}: &str = \"\\u{{{addr}}}\";\n"
                    ));
                    names.insert(name);
                    token
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    let file = PathBuf::try_from(file!()).unwrap();
    let icons = file.parent().unwrap().join("src").join("icons.rs");
    std::fs::write(icons, code).unwrap();

    println!("Generated src/icons.rs");
}
