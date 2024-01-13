#[cfg(not(target_arch = "wasm32"))]
fn main() {
    run::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("This is not supported on the web.");
}

#[cfg(not(target_arch = "wasm32"))]
mod run {
    use std::fs::File;
    use std::io::{Read, Write};

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct GalleryItem {
        id: i32,
        title: String,
        #[serde(rename = "imageUrl")]
        image_url: String,
        thumbhash: Option<Vec<u8>>,
        width: Option<f32>,
        height: Option<f32>,
    }

    pub fn run() {
        // let mut response = client.request(Request::builder(Method::GET, "https://malmal.io/api/gallery/entries?order=TOP&minDate=Thu,%2014%20Dec%202023%2001:52:57%20GMT&limit=30&offset=0".try_into().unwrap()).build()).unwrap();
        let response = ureq::get("https://malmal.io/api/gallery/entries?order=TOP&minDate=Thu,%2014%20Aug%202023%2001:52:57%20GMT&limit=100&offset=0")
            .call()
            .unwrap();

        let string = response.into_string().unwrap();

        let mut items: Vec<GalleryItem> = serde_json::from_str(&string).unwrap();

        for item in &mut items {
            // let response = client
            //     .request(
            //         Request::builder(
            //             Method::GET,
            //             Url::parse(&format!(
            //                 "https://img.malmal.io/insecure/h:300/plain/{}@jpg",
            //                 &item.image_url
            //             ))
            //             .unwrap(),
            //         )
            //         .build(),
            //     )
            //     .unwrap();
            // let bytes = response.into_body().to_vec().unwrap();

            let response = ureq::get(&format!(
                "https://img.malmal.io/insecure/h:300/plain/{}@webp",
                &item.image_url
            ))
            .call()
            .unwrap();

            let len = response.header("Content-Length").unwrap().parse().unwrap();

            let mut bytes: Vec<u8> = Vec::with_capacity(len);
            response
                .into_reader()
                .take(10_000_000)
                .read_to_end(&mut bytes)
                .unwrap();

            let mut file =
                File::create(format!("fancy-example/src/gallery/{}.webp", item.id)).unwrap();
            file.write_all(&bytes).unwrap();

            let image = image::load_from_memory(&bytes).unwrap();

            // we need to scale the image down to fit into 100x100px
            let thumbnail = image.thumbnail(100, 100);

            let thumbhash = thumbhash::rgba_to_thumb_hash(
                thumbnail.width() as usize,
                thumbnail.height() as usize,
                thumbnail.into_rgba8().as_raw(),
            );
            item.thumbhash = Some(thumbhash);
            item.width = Some(image.width() as f32);
            item.height = Some(image.height() as f32);
        }

        let json = serde_json::to_string_pretty(&items).unwrap();

        let mut index_file = File::create("fancy-example/src/gallery/index.json").unwrap();
        index_file.write_all(&json.as_bytes()).unwrap();
    }
}
