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

pub struct EguiRouter<State> {
    router: matchit::Router<Box<dyn Handler<State>>>,
    pub state: State,
    history: Vec<RouteState<State>>,
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
        } else {
            eprintln!("Failed to navigate to route: {}", route);
        }
    }

    pub fn back(&mut self) {
        self.history.pop();
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if let Some(route_state) = self.history.last_mut() {
            route_state.route.ui(ui, &mut self.state);
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
