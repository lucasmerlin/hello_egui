use std::future::Future;
use std::time::Duration;

//pub use maybe_sync::{BoxFuture, MaybeSend, MaybeSync};

#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(f: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(f);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn(f: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sleep(dur: Duration) -> impl Future<Output = ()> {
    tokio::time::sleep(dur)
}

#[cfg(target_arch = "wasm32")]
pub fn sleep(dur: Duration) -> impl Future<Output = ()> {
    gloo_timers::future::TimeoutFuture::new(dur.as_millis() as u32)
}
