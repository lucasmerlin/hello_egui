mod transition;

use crate::transition::{ActiveTransition, ActiveTransitionResult, Transition, TransitionType};
use egui::emath::ease_in_ease_out;
use egui::Ui;

pub trait Handler<State> {
    fn handle(&mut self, state: Request<State>) -> Box<dyn Route<State>>;
}

pub trait Route<State> {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State);
}

struct RouteState<State> {
    route: Box<dyn Route<State>>,
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
            _in: transition::SlideTransition::new(1.0).into(),
            out: transition::SlideTransition::new(-0.1).into(),
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

pub struct EguiRouter<State> {
    router: matchit::Router<Box<dyn Handler<State>>>,
    pub state: State,
    history: Vec<RouteState<State>>,

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

impl<State> EguiRouter<State> {
    pub fn new(state: State) -> Self {
        Self {
            router: matchit::Router::new(),
            state,
            history: Vec::new(),
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

    pub fn navigate_transition(
        &mut self,
        route: impl Into<String>,
        transition_config: TransitionConfig,
    ) {
        let route = route.into();
        let mut handler = self.router.at_mut(&route);

        if let Ok(handler) = handler {
            let route = handler.value.handle(Request {
                state: &mut self.state,
                params: handler.params,
            });
            self.history.push(RouteState { route });

            self.current_transition = Some(CurrentTransition {
                active_transition: ActiveTransition::forward(transition_config)
                    .with_default_duration(self.default_duration),
                leaving_route: None,
            });
        } else {
            eprintln!("Failed to navigate to route: {}", route);
        }
    }

    pub fn back_transition(&mut self, transition_config: TransitionConfig) {
        if self.history.len() > 1 {
            let leaving_route = self.history.pop();
            self.current_transition = Some(CurrentTransition {
                active_transition: ActiveTransition::backward(transition_config)
                    .with_default_duration(self.default_duration),
                leaving_route,
            });
        }
    }

    pub fn navigate(&mut self, route: impl Into<String>) {
        self.navigate_transition(route, self.forward_transition.clone());
    }

    pub fn back(&mut self) {
        self.back_transition(self.backward_transition.clone());
    }

    pub fn replace_transition(
        &mut self,
        route: impl Into<String>,
        transition_config: TransitionConfig,
    ) {
        let leaving_route = self.history.pop();
        let route = route.into();
        let mut handler = self.router.at_mut(&route);

        if let Ok(handler) = handler {
            let route = handler.value.handle(Request {
                state: &mut self.state,
                params: handler.params,
            });
            self.history.push(RouteState { route });

            self.current_transition = Some(CurrentTransition {
                active_transition: ActiveTransition::forward(transition_config)
                    .with_default_duration(self.default_duration),
                leaving_route,
            });
        } else {
            eprintln!("Failed to navigate to route: {}", route);
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if let Some((last, previous)) = self.history.split_last_mut() {
            let result = if let Some(transition) = &mut self.current_transition {
                let leaving_route_state = transition.leaving_route.as_mut().or(previous.last_mut());
                Some(transition.active_transition.show(
                    ui,
                    &mut self.state,
                    |ui, state| {
                        last.route.ui(ui, state);
                    },
                    leaving_route_state.map(|r| {
                        |ui: &mut _, state: &mut _| {
                            r.route.ui(ui, state);
                        }
                    }),
                ))
            } else {
                last.route.ui(ui, &mut self.state);
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
