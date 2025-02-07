# egui_webview

[![egui_ver](https://img.shields.io/badge/egui-0.31.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_webview.svg)](https://crates.io/crates/egui_webview)
[![Documentation](https://docs.rs/egui_webview/badge.svg)](https://docs.rs/egui_webview)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_webview.svg)](https://crates.io/crates/egui_webview)



[content]:<>


# egui_webview

This is a proof of concept of how a webview crate for egui could look like.
It works by taking a screenshot of the webview and rendering it as an egui texture 
whenever something is shown above the webview.

For this to work this screenshot PR on wry would be required: https://github.com/tauri-apps/wry/pull/266