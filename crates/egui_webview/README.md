# egui_webview

This is a proof of concept of how a webview crate for egui could look like.
It works by taking a screenshot of the webview and rendering it as an egui texture 
whenever something is shown above the webview.

For this to work this screenshot PR on wry would be required: https://github.com/tauri-apps/wry/pull/266