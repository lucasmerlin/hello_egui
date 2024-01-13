mod image;

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use egui::ahash::HashMap;
use egui::load::{ImageLoadResult, ImageLoader, ImagePoll, LoadError};
use egui::mutex::Mutex;
use egui::{ahash, ColorImage, Context, SizeHint};

pub use image::ThumbhashImage;

pub fn register(ctx: &Context) {
    ctx.add_image_loader(Arc::new(ThumbhashImageLoader::new()))
}

pub struct ThumbhashImageLoader {
    images: Mutex<HashMap<u64, Arc<ColorImage>>>,
}

impl ThumbhashImageLoader {
    pub fn new() -> Self {
        Self {
            images: Mutex::new(HashMap::default()),
        }
    }
}

impl ImageLoader for ThumbhashImageLoader {
    fn id(&self) -> &str {
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
                    Err(_) => Err(LoadError::Loading("Invalid thumbhash".to_string())),
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

pub fn thumbhash_to_uri(thumbhash: &[u8]) -> String {
    format!("thumbhash:{}", BASE64_STANDARD_NO_PAD.encode(thumbhash))
}

pub fn uri_to_thumbhash(uri: &str) -> Option<Vec<u8>> {
    let mut split = uri.split(":");
    let prefix = split.next()?;
    if prefix != "thumbhash" {
        return None;
    }

    let base64 = split.next()?;
    BASE64_STANDARD_NO_PAD.decode(base64.as_bytes()).ok()
}
