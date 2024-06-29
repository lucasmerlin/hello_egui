use crate::{Request, Route};
use std::fmt::Display;

#[derive(Debug)]
pub enum HandlerError {
    NotFound,
    Message(String),
    Error(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{}", msg),
            Self::NotFound => write!(f, "Handler not found"),
            Self::Error(err) => write!(f, "Handler error: {}", err),
        }
    }
}

pub type HandlerResult<T = ()> = Result<T, HandlerError>;

impl<T: std::error::Error + Send + Sync + 'static> From<T> for HandlerError {
    fn from(err: T) -> Self {
        Self::Error(Box::new(err))
    }
}

// The args argument is just so we can implement multiple specializations, like explained here:
// https://geo-ant.github.io/blog/2021/rust-traits-and-variadic-functions/
pub trait MakeHandler<State, Args> {
    fn handle(&mut self, state: Request<State>) -> HandlerResult<Box<dyn Route<State>>>;
}

pub type Handler<State> = Box<dyn FnMut(Request<State>) -> HandlerResult<Box<dyn Route<State>>>>;

impl<F, State, R> MakeHandler<State, (Request<'static, State>, ())> for F
where
    F: Fn(Request<State>) -> R,
    R: Route<State> + 'static,
{
    fn handle(&mut self, request: Request<State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self(request)))
    }
}

impl<F, State, R> MakeHandler<State, ((), ())> for F
where
    F: Fn() -> R,
    R: Route<State> + 'static,
{
    fn handle(&mut self, _request: Request<State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self()))
    }
}

impl<F, State, R> MakeHandler<State, (Request<'static, State>, HandlerResult)> for F
where
    F: Fn(Request<State>) -> HandlerResult<R>,
    R: Route<State> + 'static,
{
    fn handle(&mut self, request: Request<State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self(request)?))
    }
}

impl<F, State, R> MakeHandler<State, ((), HandlerResult)> for F
where
    F: Fn() -> HandlerResult<R>,
    R: Route<State> + 'static,
{
    fn handle(&mut self, _request: Request<State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self()?))
    }
}

#[cfg(feature = "async")]
pub mod async_impl {
    use crate::handler::HandlerResult;
    use crate::{Request, Route};
    use std::collections::BTreeMap;
    use std::future::Future;

    pub struct OwnedRequest<State> {
        pub params: BTreeMap<String, String>,
        pub state: State,
    }

    pub trait AsyncMakeHandler<State, Args> {
        fn handle(
            &self,
            state: OwnedRequest<State>,
        ) -> impl Future<Output = HandlerResult<Box<dyn Route<State> + Send + Sync>>> + Send + Sync;
    }

    impl<F, Fut, State, R> AsyncMakeHandler<State, (Request<'static, State>, ())> for F
    where
        F: Fn(OwnedRequest<State>) -> Fut + Send + Sync,
        Fut: Future<Output = R> + Send + Sync,
        R: Route<State> + 'static + Send + Sync,
        State: Send + Sync,
    {
        async fn handle(
            &self,
            request: OwnedRequest<State>,
        ) -> HandlerResult<Box<dyn Route<State> + Send + Sync>> {
            Ok(Box::new(self(request).await))
        }
    }

    impl<F, Fut, State, R> AsyncMakeHandler<State, ((), ())> for F
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = R> + Send + Sync,
        R: Route<State> + 'static + Send + Sync,
        State: Send + Sync,
    {
        async fn handle(
            &self,
            _request: OwnedRequest<State>,
        ) -> HandlerResult<Box<dyn Route<State> + Send + Sync>> {
            Ok(Box::new(self().await))
        }
    }

    impl<F, Fut, State, R> AsyncMakeHandler<State, (Request<'static, State>, HandlerResult)> for F
    where
        F: Fn(OwnedRequest<State>) -> Fut + Send + Sync,
        Fut: Future<Output = HandlerResult<R>> + Send + Sync,
        R: Route<State> + 'static + Send + Sync,
        State: Send + Sync,
    {
        async fn handle(
            &self,
            request: OwnedRequest<State>,
        ) -> HandlerResult<Box<dyn Route<State> + Send + Sync>> {
            Ok(Box::new(self(request).await?))
        }
    }

    impl<F, Fut, State, R> AsyncMakeHandler<State, ((), HandlerResult)> for F
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: Future<Output = HandlerResult<R>> + Send + Sync,
        R: Route<State> + 'static + Send + Sync,
        State: Send + Sync,
    {
        async fn handle(
            &self,
            _request: OwnedRequest<State>,
        ) -> HandlerResult<Box<dyn Route<State> + Send + Sync>> {
            Ok(Box::new(self().await?))
        }
    }
}
