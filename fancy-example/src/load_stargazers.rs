use egui::ColorImage;
use egui_extras::RetainedImage;
use ehttp::Request;
use serde::Deserialize;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub enum ImageState {
    #[default]
    None,
    Loading,
    Data(RetainedImage),
    Error(String),
}

#[derive(Deserialize)]
pub struct Stargazer {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
    #[serde(skip)]
    pub image: Arc<Mutex<ImageState>>,
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

impl Stargazer {
    pub fn load_image(&self) {
        {
            let mut guard = self.image.lock().unwrap();
            if let ImageState::None = *guard {
                *guard = ImageState::Loading;
            } else {
                return;
            }
        }

        let image_state = self.image.clone();
        let login = self.login.clone();
        let avatar_url = self.avatar_url.clone();
        ehttp::fetch(Request::get(&avatar_url), move |result| {
            if let Ok(data) = result {
                let image = RetainedImage::from_image_bytes(login, &data.bytes);

                let mut guard = image_state.lock().unwrap();
                match image {
                    Ok(image) => {
                        *guard = ImageState::Data(image);
                    }
                    Err(err) => {
                        dbg!(err);
                        *guard = ImageState::Error("Failed to load image".to_string());
                    }
                }
            }
        });
    }
}

pub fn load_stargazers(state: Arc<Mutex<StargazersState>>) {
    ehttp::fetch(
        Request::get("https://api.github.com/repos/lucasmerlin/egui_dnd/stargazers"),
        move |mut result| {
            if let Ok(data) = result {
                if let Ok(stargazers) = serde_json::from_slice::<Vec<Stargazer>>(&data.bytes) {
                    *state.lock().unwrap() = StargazersState::Data(stargazers);
                } else {
                    *state.lock().unwrap() =
                        StargazersState::Error("Failed to parse stargazers".to_string());
                }
            };
        },
    );
}

#[derive(Debug)]
pub enum StargazersState {
    None,
    Loading,
    Data(Vec<Stargazer>),
    Error(String),
}

pub type StargazersType = Arc<Mutex<StargazersState>>;
