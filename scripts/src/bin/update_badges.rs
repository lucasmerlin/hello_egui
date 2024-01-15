use std::error::Error;
use std::fs::{read_to_string, write, DirEntry};

pub const CONTENT_MARKER: &str = "[content]:#";

fn main() -> Result<(), Box<dyn Error>> {
    // list dirs under crates
    let crates = std::fs::read_dir("crates")?;

    let cargo_toml = read_to_string("Cargo.toml")?;
    let start = cargo_toml
        .split_once("\negui = { version = \"")
        .ok_or("Could not find egui version")?;
    let egui_version = start
        .1
        .split_once('"')
        .ok_or("Could not parse egui version")?
        .0;

    dbg!(egui_version);

    let template =
        read_to_string("./scripts/badges_template.md")?.replace("{{egui_version}}", egui_version);

    let mut iter = crates.into_iter();
    while let Some(Ok(item)) = iter.next() {
        let name = item.file_name();
        let name_str = name.to_str().ok_or("Invalid file name")?;
        let result = replace_example_readme(&item, &template);
        match result {
            Ok(_) => {
                println!("Successfully updated {name_str}/README.md");
            }
            Err(err) => {
                println!("Failed to update {name_str}/README.md: {err}")
            }
        }
    }

    Ok(())
}

fn replace_example_readme(item: &DirEntry, template: &str) -> Result<(), Box<dyn Error>> {
    let file_name = item.file_name();
    let crate_name = file_name
        .to_str()
        .ok_or("Failed to read crate name string")?;

    let readme_path = item.path().join("README.md");

    let readme = read_to_string(readme_path.clone())?;

    let readme = readme
        .split_once(CONTENT_MARKER)
        .map(|(_, a)| a)
        .unwrap_or(&readme)
        .trim_start();

    let template = template.replace("{{crate_name}}", crate_name);

    let out_readme = format!("{template}\n\n{CONTENT_MARKER}\n\n\n{readme}");

    write(readme_path, out_readme)?;

    Ok(())
}
