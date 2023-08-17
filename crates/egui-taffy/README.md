# Egui Taffy
This crate allows you to use the flexbox library [taffy](https://github.com/DioxusLabs/taffy) with egui.
It's currently an early prototype, so I wouldn't recommend using it in production.

It seems to be working really well. For measuring, your ui functions will be called multiple times.
Calculating the flexbox is a pretty complicated process and not very performant 
(in comparison with normal egui layout). The result is cached, but it will need to be recalculated
whenever the size of the container or a child changes, so the best usecase for this library is
in static layouts that don't change size often.  

To get started, check out the example.

SAFETY NOTE: I'm using some unsafe code to get the lifetimes working with taffy. 
I'm pretty sure what I'm doing there should be sound, but someone with more experience with
Rust safety should probably review and confirm this. 
