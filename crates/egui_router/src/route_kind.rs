use crate::{Handler, Route};

pub enum RouteKind<State> {
    Route(Box<dyn Handler<State>>),
    Redirect(String),
}
