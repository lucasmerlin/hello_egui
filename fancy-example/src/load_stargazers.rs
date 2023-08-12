use egui::ColorImage;
use egui_extras::RetainedImage;
use serde::Deserialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[derive(Deserialize)]
pub struct Stargazer {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
    #[serde(skip)]
    pub image: Arc<Mutex<Option<RetainedImage>>>,
}

impl Hash for Stargazer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.login.hash(state);
    }
}

impl Debug for Stargazer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stargazer")
            .field("login", &self.login)
            .field("html_url", &self.html_url)
            .field("avatar_url", &self.avatar_url)
            .finish()
    }
}

pub async fn load_stargazers() -> anyhow::Result<Vec<Stargazer>> {
    let response = surf::get("https://api.github.com/repos/lucasmerlin/egui_dnd/stargazers")
        .recv_json::<Vec<Stargazer>>()
        .await;

    let mut result = response.map_err(|e| anyhow::anyhow!("Failed to load stargazers: {}", e));

    if let Ok(stargazers) = &mut result {
        for stargazer in stargazers.iter_mut() {
            let image = surf::get(&stargazer.avatar_url)
                .recv_bytes()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load stargazer image: {}", e))?;

            let image = RetainedImage::from_image_bytes(stargazer.login.clone(), &image);

            if let Ok(image) = image {
                stargazer.image = Arc::new(Mutex::new(Some(image)));
            }
        }
    };

    result
}
