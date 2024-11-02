use egui::accesskit::Role;
use egui_kittest::kittest::{Node, Queryable};
use egui_kittest::Harness;
use fancy_example::chat::CHAT_EXAMPLE;
use fancy_example::example::{Example, EXAMPLES};
use fancy_example::gallery::GALLERY_EXAMPLE;
use fancy_example::stargazers::STARGAZERS_EXAMPLE;
use fancy_example::App;
use std::time::Duration;

pub fn app() -> Harness<'static> {
    let mut app = None;

    Harness::new(move |ctx| {
        let app = app.get_or_insert_with(|| App::new(ctx));
        app.show(ctx);
    })
}

fn open(example: &Example, harness: &mut Harness) {
    harness
        .get_by_role_and_name(Role::Button, example.name)
        .click();

    // TODO: Remove once run runs until no more redraws are needed
    for _ in 0..30 {
        harness.run();
    }
}

#[tokio::test]
pub async fn test_pages() {
    let mut harness = app();

    let mut errors = vec![];

    let wait = [
        (&CHAT_EXAMPLE, Some("Agreed")),
        (&GALLERY_EXAMPLE, None),
        (&STARGAZERS_EXAMPLE, Some("lucasmerlin")),
    ];

    for category in EXAMPLES {
        for example in category.examples {
            open(example, &mut harness);

            if let Some((_, wait_text)) = wait.iter().find(|(e, _)| e.slug == example.slug) {
                if let Some(text) = wait_text {
                    wait_for(&mut harness, |harness| harness.query_by_name_contains(text)).await;
                } else {
                    tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
                }
            }

            for _ in 0..10 {
                harness.run();
            }

            let res = harness.try_wgpu_snapshot(&format!("example/{}", example.name));
            if let Err(e) = res {
                errors.push(e);
            }
        }
    }

    assert!(errors.is_empty(), "Errors: {errors:#?}");
}

#[tokio::test]
pub async fn test_stargazers() {
    let mut harness = app();

    open(&STARGAZERS_EXAMPLE, &mut harness);

    wait_for(&mut harness, |harness| {
        harness.query_by_name_contains("lucasmerlin")
    })
    .await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    for _ in 0..30 {
        harness.run();
    }

    harness.wgpu_snapshot("stargazers");
}

#[tokio::test]
pub async fn test_chat() {
    let mut harness = app();

    open(&CHAT_EXAMPLE, &mut harness);

    wait_for(&mut harness, |harness| {
        harness.query_by_name_contains("Agreed!")
    })
    .await;

    harness.wgpu_snapshot("chat");
}

#[tokio::test]
pub async fn test_gallery() {
    let mut harness = app();

    open(&GALLERY_EXAMPLE, &mut harness);

    tokio::time::sleep(Duration::from_secs(1)).await;

    for _ in 0..30 {
        harness.run();
    }

    harness.wgpu_snapshot("gallery");
}

pub async fn wait_for<'h, 'hl>(
    harness: &'hl mut egui_kittest::Harness<'h>,
    query: impl for<'t> Fn(&'t Harness<'t>) -> Option<Node<'t>>,
) {
    let timeout = Duration::from_secs(2);
    let all_steps = 20;
    let mut step = all_steps;
    loop {
        harness.run();

        let result = query(harness).is_some();
        if result {
            return;
        }

        step -= 1;
        #[allow(clippy::manual_assert)]
        if step == 0 {
            panic!("Timeout exceeded while waiting for node");
        }
        tokio::time::sleep(timeout / all_steps).await;
    }
}
