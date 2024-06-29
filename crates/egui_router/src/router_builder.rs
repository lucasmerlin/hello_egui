use crate::handler::MakeHandler;
use crate::history::History;
use crate::route_kind::RouteKind;
use crate::{EguiRouter, TransitionConfig};
use std::sync::Arc;

pub type ErrorUi<State> =
    Arc<Box<dyn Fn(&mut egui::Ui, &State, &crate::handler::HandlerError) + Send + Sync>>;
pub type LoadingUi<State> = Arc<Box<dyn Fn(&mut egui::Ui, &State) + Send + Sync>>;

pub struct RouterBuilder<State, H> {
    pub(crate) router: matchit::Router<RouteKind<State>>,
    pub(crate) default_route: Option<String>,

    pub(crate) forward_transition: TransitionConfig,
    pub(crate) backward_transition: TransitionConfig,
    pub(crate) replace_transition: TransitionConfig,

    pub(crate) default_duration: Option<f32>,

    pub(crate) history_kind: Option<H>,

    pub(crate) error_ui: ErrorUi<State>,
    pub(crate) loading_ui: LoadingUi<State>,
}

impl<State: 'static, H: History + Default> Default for RouterBuilder<State, H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State: 'static, H: History + Default> RouterBuilder<State, H> {
    pub fn new() -> Self {
        Self {
            router: matchit::Router::new(),
            default_route: None,
            forward_transition: TransitionConfig::default(),
            backward_transition: TransitionConfig::default(),
            replace_transition: TransitionConfig::fade(),
            default_duration: None,
            history_kind: None,
            error_ui: Arc::new(Box::new(|ui, _, err| {
                ui.label(format!("Error: {}", err));
            })),
            loading_ui: Arc::new(Box::new(|ui, _| {
                ui.spinner();
            })),
        }
    }

    pub fn transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition.clone();
        self.backward_transition = transition;
        self
    }

    pub fn forward_transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition;
        self
    }

    pub fn backward_transition(mut self, transition: TransitionConfig) -> Self {
        self.backward_transition = transition;
        self
    }

    pub fn replace_transition(mut self, transition: TransitionConfig) -> Self {
        self.replace_transition = transition;
        self
    }

    pub fn default_duration(mut self, duration: f32) -> Self {
        self.default_duration = Some(duration);
        self
    }

    pub fn default_path(mut self, route: impl Into<String>) -> Self {
        self.default_route = Some(route.into());
        self
    }

    pub fn history(mut self, history: H) -> Self {
        self.history_kind = Some(history);
        self
    }

    /// Call this *before* you call `.async_route()`, otherwise the error UI will not be used in async routes.
    pub fn error_ui(
        mut self,
        f: impl Fn(&mut egui::Ui, &State, &crate::handler::HandlerError) + 'static + Send + Sync,
    ) -> Self {
        self.error_ui = Arc::new(Box::new(f));
        self
    }

    /// Call this *before* you call `.async_route()`, otherwise the loading UI will not be used in async routes.
    pub fn loading_ui(mut self, f: impl Fn(&mut egui::Ui, &State) + 'static + Send + Sync) -> Self {
        self.loading_ui = Arc::new(Box::new(f));
        self
    }

    pub fn route<HandlerArgs, Han: MakeHandler<State, HandlerArgs> + 'static>(
        mut self,
        route: &str,
        mut handler: Han,
    ) -> Self {
        self.router
            .insert(
                route,
                RouteKind::Route(Box::new(move |req| handler.handle(req))),
            )
            .unwrap();
        self
    }

    #[cfg(feature = "async")]
    pub fn async_route<HandlerArgs, Han>(mut self, route: &str, mut handler: Han) -> Self
    where
        Han: crate::handler::async_impl::AsyncMakeHandler<State, HandlerArgs>
            + 'static
            + Clone
            + Send
            + Sync,
        State: Clone + 'static + Send + Sync,
    {
        let loading_ui = self.loading_ui.clone();
        let error_ui = self.error_ui.clone();
        self.router
            .insert(
                route,
                RouteKind::Route(Box::new(move |mut req| {
                    let loading_ui = loading_ui.clone();
                    let error_ui = error_ui.clone();

                    let owned = crate::OwnedRequest {
                        params: req
                            .params
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        state: req.state.clone(),
                    };

                    let handler = handler.clone();

                    let state_clone = req.state.clone();
                    let state_clone2 = req.state.clone();

                    let route = crate::async_route::AsyncRoute {
                        suspense: egui_suspense::EguiSuspense::single_try_async(async move {
                            handler.handle(owned).await
                        })
                        .loading_ui(move |ui| loading_ui(ui, &state_clone))
                        .error_ui(move |ui, err, _| error_ui(ui, &state_clone2, err)),
                    };

                    Ok(Box::new(route))
                })),
            )
            .unwrap();
        self
    }

    pub fn route_redirect(mut self, route: &str, redirect: impl Into<String>) -> Self {
        self.router
            .insert(route, RouteKind::Redirect(redirect.into()))
            .unwrap();
        self
    }

    pub fn build(self, state: &mut State) -> EguiRouter<State, H> {
        EguiRouter::from_builder(self, state)
    }
}
