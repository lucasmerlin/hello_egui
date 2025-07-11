# To do the release, first git pull, then run the release command for each crate
git pull

cargo release --execute -p hello_egui_utils minor
cargo release --execute -p hello_egui_utils_dev minor
cargo release --execute -p egui_animation minor
cargo release --execute -p egui_dnd minor
cargo release --execute -p egui_inbox minor
cargo release --execute -p egui_suspense minor
cargo release --execute -p egui_virtual_list minor
cargo release --execute -p egui_infinite_scroll minor
cargo release --execute -p egui_pull_to_refresh minor
cargo release --execute -p egui_form minor
cargo release --execute -p egui_flex minor
cargo release --execute -p egui_router minor
cargo release --execute -p egui_thumbhash minor
cargo release --execute -p egui_material_icons minor
cargo release --execute -p hello_egui minor
