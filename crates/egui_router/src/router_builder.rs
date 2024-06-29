use crate::handler::MakeHandler;
use crate::history::History;
use crate::route_kind::RouteKind;
use crate::{EguiRouter, TransitionConfig};

pub struct RouterBuilder<State, H> {
    pub(crate) router: matchit::Router<RouteKind<State>>,
    pub(crate) default_route: Option<String>,

    pub(crate) forward_transition: TransitionConfig,
    pub(crate) backward_transition: TransitionConfig,
    pub(crate) replace_transition: TransitionConfig,

    pub(crate) default_duration: Option<f32>,

    pub(crate) history_kind: Option<H>,
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
        self.router
            .insert(
                route,
                RouteKind::Route(Box::new(move |req| {
                    let owned = crate::OwnedRequest {
                        params: req
                            .params
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        state: req.state.clone(),
                    };

                    let handler = handler.clone();

                    let route = crate::async_route::AsyncRoute {
                        suspense: egui_suspense::EguiSuspense::single_try_async(async move {
                            handler.handle(owned).await
                        }),
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
