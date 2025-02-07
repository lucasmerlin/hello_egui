use cargo_metadata::MetadataCommand;
use std::error::Error;
use std::fs::read_to_string;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let text = args.get(1).unwrap();

    let metadata = MetadataCommand::new().exec().unwrap();

    let packages = metadata.workspace_packages();

    for package in packages {
        if !package.name.contains("egui")
            && !package
                .dependencies
                .iter()
                .any(|dep| dep.name.contains("egui"))
        {
            continue;
        }

        let dir = package.manifest_path.parent().unwrap();
        let changelog_path = dir.join("CHANGELOG.md");

        let mut version = package.version.clone();
        version.minor += 1;

        if changelog_path.exists() {
            let mut changelog = read_to_string(&changelog_path).unwrap();

            if changelog.contains("## Unreleased") {
                changelog = changelog.replace(
                    "## Unreleased",
                    &format!(
                        "## {}

{}",
                        version, text
                    ),
                );
                std::fs::write(changelog_path, changelog)?;
            } else {
                changelog = changelog.replace(
                    &format!("## {}", package.version),
                    &format!(
                        "## {}

{}

## {}",
                        version, text, package.version
                    ),
                );
                std::fs::write(changelog_path, changelog)?;
            }
        }
    }

    Ok(())
}
