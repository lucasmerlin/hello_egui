use crate::history::History;
use crate::route_kind::RouteKind;
use crate::{EguiRouter, Handler, TransitionConfig};

pub struct RouterBuilder<State, H> {
    pub(crate) router: matchit::Router<RouteKind<State>>,
    pub(crate) default_route: Option<String>,

    pub(crate) forward_transition: TransitionConfig,
    pub(crate) backward_transition: TransitionConfig,
    pub(crate) replace_transition: TransitionConfig,

    pub(crate) default_duration: Option<f32>,

    pub(crate) history_kind: Option<H>,
}

impl<State, H: History + Default> RouterBuilder<State, H> {
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

    pub fn default_route(mut self, route: impl Into<String>) -> Self {
        self.default_route = Some(route.into());
        self
    }

    pub fn history(mut self, history: H) -> Self {
        self.history_kind = Some(history);
        self
    }

    pub fn route<Han: Handler<State> + 'static>(mut self, route: &str, handler: Han) -> Self {
        self.router
            .insert(route, RouteKind::Route(Box::new(handler)))
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
