use crate::handler::MakeHandler;
use crate::history::History;
use crate::route_kind::RouteKind;
use crate::{EguiRouter, TransitionConfig};
use std::sync::Arc;

pub(crate) type ErrorUi<State> =
    Arc<Box<dyn Fn(&mut egui::Ui, &State, &crate::handler::HandlerError) + Send + Sync>>;
pub(crate) type LoadingUi<State> = Arc<Box<dyn Fn(&mut egui::Ui, &State) + Send + Sync>>;

/// Builder to create a [`EguiRouter`]
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
    /// Create a new router builder
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
                ui.label(format!("Error: {err}"));
            })),
            loading_ui: Arc::new(Box::new(|ui, _| {
                ui.spinner();
            })),
        }
    }

    /// Set the transition for both forward and backward transitions
    pub fn transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition.clone();
        self.backward_transition = transition;
        self
    }

    /// Set the transition for forward transitions
    pub fn forward_transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition;
        self
    }

    /// Set the transition for backward transitions
    pub fn backward_transition(mut self, transition: TransitionConfig) -> Self {
        self.backward_transition = transition;
        self
    }

    /// Set the transition for replace transitions
    pub fn replace_transition(mut self, transition: TransitionConfig) -> Self {
        self.replace_transition = transition;
        self
    }

    /// Set the default duration for transitions
    pub fn default_duration(mut self, duration: f32) -> Self {
        self.default_duration = Some(duration);
        self
    }

    /// Set the default route (when using [`history::BrowserHistory`], window.location.pathname will be used instead)
    pub fn default_path(mut self, route: impl Into<String>) -> Self {
        self.default_route = Some(route.into());
        self
    }

    /// Set the history implementation
    pub fn history(mut self, history: H) -> Self {
        self.history_kind = Some(history);
        self
    }

    /// Set the error UI
    /// Call this *before* you call `.async_route()`, otherwise the error UI will not be used in async routes.
    pub fn error_ui(
        mut self,
        f: impl Fn(&mut egui::Ui, &State, &crate::handler::HandlerError) + 'static + Send + Sync,
    ) -> Self {
        self.error_ui = Arc::new(Box::new(f));
        self
    }

    /// Set the loading UI
    /// Call this *before* you call `.async_route()`, otherwise the loading UI will not be used in async routes.
    pub fn loading_ui(mut self, f: impl Fn(&mut egui::Ui, &State) + 'static + Send + Sync) -> Self {
        self.loading_ui = Arc::new(Box::new(f));
        self
    }

    /// Add a route. Check the [matchit] documentation for information about the route syntax.
    /// The handler will be called with [`crate::Request`] and should return a [Route].
    ///
    /// # Example
    /// ```rust
    /// # use egui::Ui;
    /// # use egui_router::{EguiRouter, HandlerError, HandlerResult, Request, Route};
    ///
    /// pub fn my_handler(_req: Request) -> impl Route {
    ///     |ui: &mut Ui, _: &mut ()| {
    ///         ui.label("Hello, world!");
    ///     }
    /// }
    ///
    /// pub fn my_fallible_handler(req: Request) -> HandlerResult<impl Route> {
    ///     let post = req.params.get("post").ok_or_else(|| HandlerError::NotFound)?.to_owned();
    ///     Ok(move |ui: &mut Ui, _: &mut ()| {
    ///        ui.label(format!("Post: {}", post));
    ///     })
    /// }
    ///
    /// let router: EguiRouter<()> = EguiRouter::builder()
    ///     .route("/", my_handler)
    ///     .route("/:post", my_fallible_handler)
    ///     .build(&mut ());
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

    /// Add an async route. Check the [matchit] documentation for information about the route syntax.
    /// The handler will be called with [`crate::OwnedRequest`] and should return a [Route].
    ///
    /// # Example
    /// ```rust
    /// # use egui::Ui;
    /// # use egui_router::{EguiRouter, HandlerError, HandlerResult, Request, Route};
    /// # #[cfg(feature = "async")]
    /// async fn my_handler(_req: egui_router::OwnedRequest) -> HandlerResult<impl Route> {
    ///    Ok(move |ui: &mut Ui, _: &mut ()| {
    ///       ui.label("Hello, world!");
    ///    })
    /// }
    ///
    /// # #[cfg(feature = "async")]
    /// async fn my_fallible_handler(req: egui_router::OwnedRequest) -> HandlerResult<impl Route> {
    ///     let post = req.params.get("post").ok_or_else(|| HandlerError::NotFound)?.to_owned();
    ///     Ok(move |ui: &mut Ui, _: &mut ()| {
    ///         ui.label(format!("Post: {}", post));
    ///     })
    /// }
    ///
    /// # #[cfg(feature = "async")]
    /// let router: EguiRouter<()> = EguiRouter::builder()
    ///    .async_route("/", my_handler)
    ///    .async_route("/:post", my_fallible_handler)
    ///    .build(&mut ());
    #[cfg(feature = "async")]
    pub fn async_route<HandlerArgs, Han>(mut self, route: &str, handler: Han) -> Self
    where
        Han: crate::handler::AsyncMakeHandler<State, HandlerArgs> + 'static + Clone + Send + Sync,
        State: Clone + 'static + Send + Sync,
    {
        let loading_ui = self.loading_ui.clone();
        let error_ui = self.error_ui.clone();
        self.router
            .insert(
                route,
                RouteKind::Route(Box::new(move |req| {
                    let loading_ui = loading_ui.clone();
                    let error_ui = error_ui.clone();

                    let owned = crate::OwnedRequest {
                        params: req
                            .params
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        query: req
                            .query
                            .into_iter()
                            .map(|(k, v)| (k.into_owned(), v.into_owned()))
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

    /// Add a redirect route. Whenever this route matches, it'll redirect to the route you specified.
    pub fn route_redirect(mut self, route: &str, redirect: impl Into<String>) -> Self {
        self.router
            .insert(route, RouteKind::Redirect(redirect.into()))
            .unwrap();
        self
    }

    /// Build the router
    pub fn build(self, state: &mut State) -> EguiRouter<State, H> {
        EguiRouter::from_builder(self, state)
    }
}
