#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::{Align, Label, Layout, Ui, Vec2, WidgetText};
use std::future::Future;

/// Returns the size of the text in the current ui (based on the max width of the ui)
pub fn measure_text(ui: &mut Ui, text: impl Into<WidgetText>) -> Vec2 {
    // There might be a more elegant way but this is enough for now
    let res = Label::new(text).layout_in_ui(&mut ui.child_ui(
        ui.available_rect_before_wrap(),
        Layout::left_to_right(Align::Center),
    ));

    // There seem to be rounding errors in egui's text rendering
    // so we add a little bit of padding
    res.2.rect.size() + Vec2::new(0.1, 0.0)
}

/// Returns the approximate current scroll delta of the ui
pub fn current_scroll_delta(ui: &Ui) -> Vec2 {
    -ui.min_rect().min.to_vec2()
}

#[cfg(feature = "tokio")]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    tokio::task::spawn(future);
}

#[macro_export]
macro_rules! async_callback {
    (
        $(#[$outer:meta])*
        $vis:vis fn $name:ident $async_name:ident($f:ident: CallbackOnce<$result:ty> $(, $arg:ident: $arg_ty:ty)*) -> $ret:ty $body:block
    ) => {
        $(#[$outer])*
        $vis fn $name($($arg: $arg_ty),* $f: impl FnOnce($crate::CallbackFn<$result>) + 'static + Send + Sync) -> $ret {
            $body
        }

        #[cfg(feature = "async")]
        $(#[$outer])*
        #[doc = concat!("This is the async version of [", stringify!($name), "]")]
        $vis fn $async_name($($arg: $arg_ty),* future: impl Future<Output = $result> + 'static + Send + Sync) -> $ret {
            Self::$name(move |__callback| {
                ::hello_egui_utils::spawn(async move {
                    __callback(future.await);
                });
            })
        }
    };
    (
        $(#[$outer:meta])*
        $vis:vis fn $name:ident $async_name:ident(mut $f:ident: CallbackMut<$result:ty> $(, $arg:ident: $arg_ty:ty)*) -> $ret:ty $body:block
    ) => {
        $(#[$outer])*
        $vis fn $name($($arg: $arg_ty),* mut $f: impl FnMut($crate::CallbackFn<$result>) + 'static + Send + Sync) -> $ret {
            $body
        }

        #[cfg(feature = "async")]
        $(#[$outer])*
        #[doc = concat!("This is the async version of [", stringify!($name), "]")]
        $vis fn $async_name<F: Future<Output = $result> + 'static + Send + Sync>($($arg: $arg_ty),* mut future: impl FnMut() -> F + 'static + Send + Sync) -> $ret {
            Self::$name(move |__callback| {
                let mut future = future();
                ::hello_egui_utils::spawn(async move {
                    __callback(future.await);
                });
            })
        }
    };
}

pub struct Test<T, E> {
    _t: std::marker::PhantomData<T>,
    _e: std::marker::PhantomData<E>,
}

pub type CallbackFn<T> = Box<dyn FnOnce(T) + Send>;

type CallbackOnce<T> = Box<CallbackFn<T>>;

// impl<T: Send + 'static, E: Send + 'static> Test<T, E> {
//     async_callback!(
//         pub fn single_try single_try_async(f: CallbackOnce<Result<T, E>> ) -> std::sync::mpsc::Receiver<Result<T, E>> {
//             let (tx, rx) = std::sync::mpsc::channel();
//
//             f(Box::new(move |result| {
//                 tx.send(result).ok();
//             }));
//
//             rx
//         }
//     );
// }
