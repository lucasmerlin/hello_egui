use crate::handler::Handler;

pub enum RouteKind<State> {
    Route(Handler<State>),
    Redirect(String),
}
