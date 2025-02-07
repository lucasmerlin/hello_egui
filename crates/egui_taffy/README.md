# egui_taffy

[![egui_ver](https://img.shields.io/badge/egui-0.31.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_taffy.svg)](https://crates.io/crates/egui_taffy)
[![Documentation](https://docs.rs/egui_taffy/badge.svg)](https://docs.rs/egui_taffy)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_taffy.svg)](https://crates.io/crates/egui_taffy)



[content]:<>


# egui_taffy

This crate allows you to use the flexbox library [taffy](https://github.com/DioxusLabs/taffy) with egui.
It's currently an early prototype, so I wouldn't recommend using it in production.
Also it's based on [WIP changes in taffy](https://github.com/DioxusLabs/taffy/pull/490), a release of 
this crate depends on those changes.

It seems to be working really well. For measuring, your ui functions will be called multiple times.
Calculating the flexbox is a pretty complicated process and not very performant 
(in comparison with normal egui layout). The result is cached, but it will need to be recalculated
whenever the size of the container or a child changes, so the best usecase for this library is
in static layouts that don't change size often.  

To get started, check out the example.
