pub mod history;
pub mod transition;

use crate::history::{History, HistoryError, HistoryEvent};
use crate::transition::{
    ActiveTransition, ActiveTransitionResult, SlideFadeTransition, SlideTransition, Transition,
    TransitionType,
};
use egui::emath::ease_in_ease_out;
use egui::{Id, Ui, Vec2};
use matchit::MatchError;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait Handler<State> {
    fn handle(&mut self, state: Request<State>) -> Box<dyn Route<State>>;
}

pub trait Route<State> {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State);
}

static ID: AtomicUsize = AtomicUsize::new(0);

struct RouteState<State> {
    path: String,
    route: Box<dyn Route<State>>,
    id: usize,
    state: u32,
}

pub type RouterResult<T = ()> = Result<T, RouterError>;

#[derive(Debug)]
pub enum RouterError {
    HistoryError(HistoryError),
    NotFound,
}

impl From<HistoryError> for RouterError {
    fn from(err: HistoryError) -> Self {
        Self::HistoryError(err)
    }
}

#[derive(Debug, Clone)]
pub struct TransitionConfig {
    duration: Option<f32>,
    easing: fn(f32) -> f32,
    _in: Transition,
    out: Transition,
}

impl Default for TransitionConfig {
    fn default() -> Self {
        Self {
            duration: None,
            easing: ease_in_ease_out,
            _in: transition::SlideTransition::new(Vec2::X).into(),
            out: transition::SlideTransition::new(Vec2::X * -0.1).into(),
        }
    }
}

impl TransitionConfig {
    pub fn new(_in: impl Into<Transition>, out: impl Into<Transition>) -> Self {
        Self {
            _in: _in.into(),
            out: out.into(),
            ..Self::default()
        }
    }

    pub fn slide() -> Self {
        Self::default()
    }

    pub fn fade_up() -> Self {
        Self::new(
            SlideFadeTransition(
                SlideTransition::new(Vec2::Y * 0.3),
                transition::FadeTransition,
            ),
            transition::NoTransition,
        )
    }

    pub fn fade() -> Self {
        Self::new(transition::FadeTransition, transition::FadeTransition)
    }

    pub fn none() -> Self {
        Self::new(transition::NoTransition, transition::NoTransition)
    }

    pub fn with_easing(mut self, easing: fn(f32) -> f32) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = Some(duration);
        self
    }
}

struct CurrentTransition<State> {
    active_transition: ActiveTransition,
    leaving_route: Option<RouteState<State>>,
}

#[cfg(target_arch = "wasm32")]
pub type DefaultHistory = history::BrowserHistory;
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultHistory = history::MemoryHistory;

pub struct EguiRouter<State, History = DefaultHistory> {
    router: matchit::Router<Box<dyn Handler<State>>>,
    history: Vec<RouteState<State>>,

    history_kind: History,

    state: usize,

    forward_transition: TransitionConfig,
    backward_transition: TransitionConfig,
    replace_transition: TransitionConfig,

    current_transition: Option<CurrentTransition<State>>,
    default_duration: Option<f32>,
}

pub struct Request<'a, State = ()> {
    pub params: matchit::Params<'a, 'a>,
    pub state: &'a mut State,
}

impl<State, H: History + Default> EguiRouter<State, H> {
    pub fn new() -> Self {
        Self {
            router: matchit::Router::new(),
            history: Vec::new(),
            history_kind: H::default(),
            state: 0,
            // default_transition: transition::Transition::Fade(transition::FadeTransition),
            current_transition: None,
            forward_transition: TransitionConfig::default(),
            backward_transition: TransitionConfig::default(),
            replace_transition: TransitionConfig::fade(),
            default_duration: None,
        }
    }

    pub fn with_transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition.clone();
        self.backward_transition = transition;
        self
    }

    pub fn with_forward_transition(mut self, transition: TransitionConfig) -> Self {
        self.forward_transition = transition;
        self
    }

    pub fn with_backward_transition(mut self, transition: TransitionConfig) -> Self {
        self.backward_transition = transition;
        self
    }

    pub fn with_replace_transition(mut self, transition: TransitionConfig) -> Self {
        self.replace_transition = transition;
        self
    }

    pub fn with_default_duration(mut self, duration: f32) -> Self {
        self.default_duration = Some(duration);
        self
    }

    pub fn active_route(&self) -> Option<&str> {
        self.history.last().map(|r| r.path.as_str())
    }

    pub fn route(
        mut self,
        route: impl Into<String>,
        handler: impl Handler<State> + 'static,
    ) -> Self {
        self.router
            .insert(route.into(), Box::new(handler))
            .expect("Invalid route");
        self
    }

    fn navigate_impl(
        &mut self,
        state: &mut State,
        path: String,
        transition_config: TransitionConfig,
        new_state: u32,
    ) -> RouterResult {
        let mut handler = self.router.at_mut(&path);

        match handler {
            Ok(handler) => {
                let route = handler.value.handle(Request {
                    state,
                    params: handler.params,
                });
                self.history.push(RouteState {
                    path,
                    route,
                    id: ID.fetch_add(1, Ordering::SeqCst),
                    state: new_state,
                });

                self.current_transition = Some(CurrentTransition {
                    active_transition: ActiveTransition::forward(transition_config)
                        .with_default_duration(self.default_duration),
                    leaving_route: None,
                });

                Ok(())
            }
            Err(e) => match e {
                MatchError::NotFound => Err(RouterError::NotFound),
            },
        }
    }

    pub fn navigate_transition(
        &mut self,
        state: &mut State,
        path: impl Into<String>,
        transition_config: TransitionConfig,
    ) -> RouterResult {
        let path = path.into();
        let current_state = self.history.last().map(|r| r.state).unwrap_or(0);
        let new_state = current_state + 1;
        self.navigate_impl(state, path.clone(), transition_config, new_state)?;
        self.history_kind.push(&path, new_state)?;
        Ok(())
    }

    fn back_impl(&mut self, transition_config: TransitionConfig) -> RouterResult {
        if self.history.len() > 1 {
            let leaving_route = self.history.pop();
            self.current_transition = Some(CurrentTransition {
                active_transition: ActiveTransition::backward(transition_config)
                    .with_default_duration(self.default_duration),
                leaving_route,
            });
        }
        Ok(())
    }

    pub fn back_transition(&mut self, transition_config: TransitionConfig) -> RouterResult {
        self.history_kind.back()?;
        self.back_impl(transition_config)
    }

    pub fn navigate(&mut self, state: &mut State, route: impl Into<String>) -> RouterResult {
        self.navigate_transition(state, route, self.forward_transition.clone())
    }

    pub fn back(&mut self) -> RouterResult {
        self.back_transition(self.backward_transition.clone())
    }

    pub fn replace_transition(
        &mut self,
        state: &mut State,
        path: impl Into<String>,
        transition_config: TransitionConfig,
    ) -> RouterResult {
        let path = path.into();
        let handler = self.router.at_mut(&path);

        let current_state = self.history.last().map(|r| r.state).unwrap_or(0);
        let new_state = current_state;

        match handler {
            Ok(handler) => {
                self.history_kind.replace(&path, new_state)?;
                let leaving_route = self.history.pop();
                let route = handler.value.handle(Request {
                    state,
                    params: handler.params,
                });
                self.history.push(RouteState {
                    path,
                    route,
                    id: ID.fetch_add(1, Ordering::SeqCst),
                    state: new_state,
                });

                self.current_transition = Some(CurrentTransition {
                    active_transition: ActiveTransition::forward(transition_config)
                        .with_default_duration(self.default_duration),
                    leaving_route,
                });

                Ok(())
            }
            Err(MatchError::NotFound) => Err(RouterError::NotFound),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, state: &mut State) {
        for e in self.history_kind.update(ui.ctx()) {
            web_sys::console::log_1(&format!("{:?}", e).into());
            let state_index = e.state.unwrap_or(0);
            let path = e.location;

            if let Some(route_state) = self
                .history
                .iter()
                .find(|r| r.path == path && r.state == state_index)
                .map(|r| r.state)
            {
                let active_state = self.history.last().map(|r| r.state).unwrap_or(0);

                // Retain all routes with a state less than or equal to the new state and the active state so that we can animate them out
                self.history
                    .retain(|r| r.state <= route_state || r.state == active_state);

                self.back_impl(self.backward_transition.clone()).ok();
            } else {
                self.navigate_impl(state, path, self.forward_transition.clone(), state_index)
                    .ok();
            }
        }

        if let Some((last, previous)) = self.history.split_last_mut() {
            let result = if let Some(transition) = &mut self.current_transition {
                let leaving_route_state = transition.leaving_route.as_mut().or(previous.last_mut());
                Some(transition.active_transition.show(
                    ui,
                    state,
                    (last.id, |ui, state| {
                        last.route.ui(ui, state);
                    }),
                    leaving_route_state.map(|r| {
                        (r.id, |ui: &mut Ui, state: &mut _| {
                            r.route.ui(ui, state);
                        })
                    }),
                ))
            } else {
                ActiveTransition::show_default(ui, last.id, |ui| {
                    last.route.ui(ui, state);
                });
                None
            };

            match result {
                Some(ActiveTransitionResult::Done) => {
                    self.current_transition = None;
                }
                Some(ActiveTransitionResult::Continue) | None => {}
            }
        }
    }
}

impl<F, State, R: Route<State> + 'static> Handler<State> for F
where
    F: Fn(Request<State>) -> R,
{
    fn handle(&mut self, request: Request<State>) -> Box<dyn Route<State>> {
        Box::new(self(request))
    }
}

// impl<F, Fut, State, R: 'static> Handler<State> for F
// where
//     F: Fn(&mut State) -> Fut,
//     Fut: std::future::Future<Output = R>,
// {
//     async fn handle(&mut self, state: &mut State) -> Box<dyn Route<State>> {
//         Box::new((self(state)).await)
//     }
// }

impl<F: FnMut(&mut Ui, &mut State), State> Route<State> for F {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State) {
        self(ui, state)
    }
}
