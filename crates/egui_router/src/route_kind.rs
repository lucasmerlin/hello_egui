use crate::handler::Handler;

pub(crate) enum RouteKind<State> {
    Route(Handler<State>),
    Redirect(String),
}
