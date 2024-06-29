use crate::history::{History, HistoryEvent, HistoryResult};
use egui_inbox::UiInbox;
use js_sys::Number;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::window;

pub struct BrowserHistory {
    base_href: String,
    inbox: UiInbox<HistoryEvent>,
    history: web_sys::History,
    closure: Closure<dyn FnMut(web_sys::PopStateEvent)>,
}

impl Default for BrowserHistory {
    fn default() -> Self {
        Self::new(None)
    }
}

impl BrowserHistory {
    pub fn new(base_href: Option<String>) -> Self {
        let window = window().unwrap();

        let base_href = base_href.unwrap_or_else(|| {
            window
                .document()
                .unwrap()
                .get_elements_by_tag_name("base")
                .item(0)
                .map(|base| base.get_attribute("href").unwrap_or_default())
                .unwrap_or_default()
        });

        let (tx, inbox) = UiInbox::channel();

        let base_href_clone = base_href.clone();
        let cb = Closure::wrap(Box::new(move |event: web_sys::PopStateEvent| {
            let state = event.state().as_f64().map(|n| n as u32);
            let location = web_sys::window().unwrap().location();
            let path = format!(
                "{}{}{}",
                location.pathname().unwrap(),
                location.search().unwrap(),
                location.hash().unwrap()
            );
            tx.send(HistoryEvent {
                location: path.trim_start_matches(&base_href_clone).to_string(),
                state,
            })
            .ok();
        }) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("popstate", cb.as_ref().unchecked_ref())
            .unwrap();
        Self {
            base_href,
            inbox,
            history: window.history().unwrap(),
            closure: cb,
        }
    }
}

impl Drop for BrowserHistory {
    fn drop(&mut self) {
        window()
            .unwrap()
            .remove_event_listener_with_callback("popstate", self.closure.as_ref().unchecked_ref())
            .unwrap();
    }
}

impl History for BrowserHistory {
    fn update(&mut self, ctx: &egui::Context) -> impl Iterator<Item = HistoryEvent> + 'static {
        self.inbox.read(ctx)
    }

    fn active_route(&self) -> Option<(String, Option<u32>)> {
        let location = window().unwrap().location();
        let full_path = format!(
            "{}{}{}",
            location.pathname().unwrap(),
            location.search().unwrap(),
            location.hash().unwrap(),
        );
        let path = if self.base_href.starts_with(&full_path) {
            "/".to_string()
        } else {
            full_path.trim_start_matches(&self.base_href).to_string()
        };
        let state = self
            .history
            .state()
            .ok()
            .map(|s| s.as_f64())
            .flatten()
            .map(|n| n as u32);
        Some((path, state))
    }

    fn push(&mut self, url: &str, state: u32) -> HistoryResult {
        self.history.push_state_with_url(
            &Number::from(state),
            "",
            Some(&format!("{}{}", self.base_href, url)),
        )?;
        Ok(())
    }

    fn replace(&mut self, url: &str, state: u32) -> HistoryResult {
        self.history.replace_state_with_url(
            &Number::from(state),
            "",
            Some(&format!("{}{}", self.base_href, url)),
        )?;
        Ok(())
    }

    fn back(&mut self) -> HistoryResult {
        self.history.back()?;
        Ok(())
    }

    fn forward(&mut self) -> HistoryResult {
        self.history.forward()?;
        Ok(())
    }
}
