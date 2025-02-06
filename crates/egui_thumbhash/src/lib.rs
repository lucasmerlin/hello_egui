#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use egui::ahash::HashMap;
use egui::load::{ImageLoadResult, ImageLoader, ImagePoll, LoadError};
use egui::mutex::Mutex;
use egui::{ahash, ColorImage, Context, SizeHint};

pub use image::ThumbhashImage;

mod image;

/// Register the thumbhash image loader with the given egui context.
/// Do this once while the app is initializing.
pub fn register(ctx: &Context) {
    ctx.add_image_loader(Arc::new(ThumbhashImageLoader::new()));
}

/// The `ImageLoader` implementation for thumbhash images.
#[derive(Clone, Default)]
pub struct ThumbhashImageLoader {
    images: Mutex<HashMap<u64, Arc<ColorImage>>>,
}

impl ThumbhashImageLoader {
    /// Create a new `ThumbhashImageLoader`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ImageLoader for ThumbhashImageLoader {
    fn id(&self) -> &'static str {
        "thumbhash"
    }

    fn load(&self, _ctx: &Context, uri: &str, _size_hint: SizeHint) -> ImageLoadResult {
        if let Some(bytes) = uri_to_thumbhash(uri) {
            let mut hasher = ahash::AHasher::default();

            bytes.hash(&mut hasher);

            let id = hasher.finish();

            let mut images = self.images.lock();

            let image = images.get(&id).cloned();

            if let Some(image) = image {
                Ok(ImagePoll::Ready { image })
            } else {
                let result = thumbhash::thumb_hash_to_rgba(&bytes);

                match result {
                    Ok((w, h, vec)) => {
                        let image = Arc::new(ColorImage::from_rgba_unmultiplied([w, h], &vec));
                        images.insert(id, image.clone());
                        Ok(ImagePoll::Ready { image })
                    }
                    Err(()) => Err(LoadError::Loading("Invalid thumbhash".to_string())),
                }
            }
        } else {
            Err(LoadError::NotSupported)
        }
    }

    fn forget(&self, uri: &str) {
        if let Some(bytes) = uri_to_thumbhash(uri) {
            let mut hasher = ahash::AHasher::default();
            bytes.hash(&mut hasher);
            let id = hasher.finish();
            let mut images = self.images.lock();
            images.remove(&id);
        }
    }

    fn forget_all(&self) {
        let mut images = self.images.lock();
        images.clear();
    }

    fn byte_size(&self) -> usize {
        let images = self.images.lock();
        images
            .iter()
            .map(|(_, image)| image.width() * image.height() * 4)
            .sum()
    }
}

/// Convert a thumbhash to a URI that can be loaded by the image loader.
pub fn thumbhash_to_uri(thumbhash: &[u8]) -> String {
    format!("thumbhash:{}", BASE64_STANDARD_NO_PAD.encode(thumbhash))
}

/// Get the thumbhash data from the thumbhash URI.
/// Returns None if the URI is not a valid thumbhash URI.
/// Will not check whether the thumbhash is valid.
pub fn uri_to_thumbhash(uri: &str) -> Option<Vec<u8>> {
    if let Some((prefix, base64)) = uri.split_once(':') {
        if prefix != "thumbhash" {
            return None;
        }
        BASE64_STANDARD_NO_PAD.decode(base64.as_bytes()).ok()
    } else {
        None
    }
}
