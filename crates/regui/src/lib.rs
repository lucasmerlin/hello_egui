#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod backend;
mod input;
mod output;
mod transform;
mod viewport;

#[cfg(feature = "wgpu")]
mod backdrop;
#[cfg(feature = "wgpu")]
mod wgpu_state;

pub use transform::Transform;
pub use viewport::{Regui, ReguiOutput};

#[cfg(feature = "wgpu")]
pub use backdrop::{BackdropBlur, PendingBlur};
#[cfg(feature = "wgpu")]
pub use wgpu_state::install_wgpu;
