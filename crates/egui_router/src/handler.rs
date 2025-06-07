use crate::{Request, Route};

/// Error returned from a [Handler]
#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    /// Not found error
    #[error("Page not found")]
    NotFound,
    /// Custom error message
    #[error("{0}")]
    Message(String),
    /// Boxed error
    #[error("Handler error: {0}")]
    Boxed(Box<dyn std::error::Error + Send + Sync>),
}

/// Handler Result type
pub type HandlerResult<T = ()> = Result<T, HandlerError>;

/// Trait for a route handler.
// The args argument is just so we can implement multiple specializations, like explained here:
// https://geo-ant.github.io/blog/2021/rust-traits-and-variadic-functions/
pub trait MakeHandler<State, Args> {
    fn handle(&mut self, state: Request<'_, State>) -> HandlerResult<Box<dyn Route<State>>>;
}

pub(crate) type Handler<State> =
    Box<dyn FnMut(Request<'_, State>) -> HandlerResult<Box<dyn Route<State>>>>;

impl<F, State, R> MakeHandler<State, (Request<'static, State>, ())> for F
where
    F: Fn(Request<'_, State>) -> R,
    R: Route<State> + 'static,
{
    fn handle(&mut self, request: Request<'_, State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self(request)))
    }
}

impl<F, State, R> MakeHandler<State, ((), ())> for F
where
    F: Fn() -> R,
    R: Route<State> + 'static,
{
    fn handle(&mut self, _request: Request<'_, State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self()))
    }
}

impl<F, State, R> MakeHandler<State, (Request<'static, State>, HandlerResult)> for F
where
    F: Fn(Request<'_, State>) -> HandlerResult<R>,
    R: Route<State> + 'static,
{
    fn handle(&mut self, request: Request<'_, State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self(request)?))
    }
}

impl<F, State, R> MakeHandler<State, ((), HandlerResult)> for F
where
    F: Fn() -> HandlerResult<R>,
    R: Route<State> + 'static,
{
    fn handle(&mut self, _request: Request<'_, State>) -> HandlerResult<Box<dyn Route<State>>> {
        Ok(Box::new(self()?))
    }
}

#[cfg(feature = "async")]
mod async_impl {
    use crate::handler::HandlerResult;
    use crate::{OwnedRequest, Request, Route};
    use std::future::Future;

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

#[cfg(feature = "async")]
pub use async_impl::*;
