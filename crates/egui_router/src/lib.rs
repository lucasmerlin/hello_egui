mod transition;

use crate::transition::{ActiveTransition, ActiveTransitionResult, Transition, TransitionType};
use egui::Ui;

pub trait Handler<State> {
    fn handle(&mut self, state: &mut Request<State>) -> Box<dyn Route<State>>;
}

pub trait Route<State> {
    fn ui(&mut self, ui: &mut egui::Ui, state: &mut State);
}

struct RouteState<State> {
    route: Box<dyn Route<State>>,
}

struct TransitionConfig {
    duration: f32,
    easing: fn(f32) -> f32,
    _in: Transition,
    out: Transition,
}

pub struct EguiRouter<State> {
    router: matchit::Router<Box<dyn Handler<State>>>,
    pub state: State,
    history: Vec<RouteState<State>>,

    current_transition: Option<ActiveTransition>,
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
        }
    }

    pub fn route(&mut self, route: impl Into<String>, handler: impl Handler<State> + 'static) {
        self.router
            .insert(route.into(), Box::new(handler))
            .expect("Invalid route");
    }

    pub fn navigate(&mut self, route: impl Into<String>) {
        let route = route.into();
        let mut handler = self.router.at_mut(&route);

        if let Ok(handler) = handler {
            let route = handler.value.handle(&mut Request {
                state: &mut self.state,
                params: handler.params,
            });
            self.history.push(RouteState { route });

            self.current_transition = Some(ActiveTransition::new(
                0.9,
                TransitionType::Forward {
                    _in: transition::Transition::Slide(transition::SlideTransition),
                    out: transition::Transition::NoTransition(transition::NoTransition),
                },
            ));
        } else {
            eprintln!("Failed to navigate to route: {}", route);
        }
    }

    pub fn back(&mut self) {
        if self.history.len() > 1 {
            self.current_transition = Some(ActiveTransition::new(
                0.9,
                TransitionType::Backward {
                    _in: transition::Transition::Slide(transition::SlideTransition),
                    out: transition::Transition::NoTransition(transition::NoTransition),
                },
            ));
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if let Some((last, previous)) = self.history.split_last_mut() {
            let result = if let Some(transition) = &mut self.current_transition {
                let transition_result = transition.update(ui.input(|i| i.stable_dt));

                transition.show(
                    ui,
                    &mut self.state,
                    |ui, state| {
                        last.route.ui(ui, state);
                    },
                    previous.last_mut().map(|r| {
                        |ui: &mut _, state: &mut _| {
                            r.route.ui(ui, state);
                        }
                    }),
                );

                Some(transition_result)
            } else {
                last.route.ui(ui, &mut self.state);
                None
            };

            match result {
                Some(ActiveTransitionResult::Done) => {
                    self.current_transition = None;
                }
                Some(ActiveTransitionResult::DonePop) => {
                    self.current_transition = None;
                    self.history.pop();
                }
                Some(ActiveTransitionResult::Continue) | None => {}
            }
        }
    }
}

impl<F, State, R: Route<State> + 'static> Handler<State> for F
where
    F: Fn(&mut Request<State>) -> R,
{
    fn handle(&mut self, request: &mut Request<State>) -> Box<dyn Route<State>> {
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
