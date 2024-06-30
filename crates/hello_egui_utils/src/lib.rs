#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Helper struct to easily align things with unknown sizes
pub mod center;

use egui::{Align, Label, Layout, Ui, Vec2, WidgetText};

pub use concat_idents::concat_idents;

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

/// Spawns a tokio task
#[cfg(all(feature = "tokio", not(target_arch = "wasm32")))]
pub fn spawn(future: impl std::future::Future<Output = ()> + Send + 'static) {
    tokio::task::spawn(future);
}

/// Spawns a wasm_bindgen_futures task
#[cfg(all(feature = "async", target_arch = "wasm32"))]
pub fn spawn(future: impl std::future::Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(all(feature = "async", not(feature = "tokio"), not(target_arch = "wasm32")))]
compile_error!("You need to enable the `tokio` feature to use this crate on native. If you need a different async runtime, please open an issue (should be easy to add).");

/// Matches the self reference of the callback fn
#[macro_export]
macro_rules! call_self_fn {
    ($path:path, &mut $self:expr, ($($arg:expr,)*)) => {
        $path($self, $($arg,)*)
    };
    ($path:path, mut $self:expr, ($($arg:expr,)*)) => {
        $path($self, $($arg,)*)
    };
    ($path:path, &$self:expr, ($($arg:expr,)*)) => {
        $path($self, $($arg,)*)
    };
    ($path:path, $self:expr, ($($arg:expr,)*)) => {
        $path($self, $($arg,)*)
    };
    ($path:path, ($($arg:expr,)*)) => {
        $path($($arg,)*)
    };
}

/// This macro generates the async fn
#[macro_export]
macro_rules! async_fn_def {
    (
        $(#[$docs:meta])*
        // Block that contains the async logic
        $body:block,
        // The name of the async fn
        $name:ident,
        // The path to the callback fn (e.g. Self::callback)
        $callback_fn_path:path,
        // The parameters of the async fn. The semicolon in front of the mutt is a hack to circumvent ambiguity
        ($($(;$mutt:ident)? $arg:ident: $typ:ty,)*)
        // The parameters of the call to the callback fn
        ($($call_args:ident,)*)
        // The generics of the async fn
        ($($gen:tt)*),
        // Return type
        ($($return_type:tt)*),
        // Self reference
        ($($callback_body_self:tt)*),
    ) => {
        // We use concat_idents to generate the name for the async fn
        $crate::concat_idents!(fn_name = $name, _async {
            $(#[$docs])*
            #[doc = concat!("This is the async version of `", stringify!($name), "`")]
            #[allow(unused_mut)]
            pub fn fn_name$($gen)*(
                $($callback_body_self)*
                $($($mutt)? $arg: $typ,)*
            ) -> $($return_type)* {
                let callback = $body;

                // Construct the call to the callback fn
                $crate::call_self_fn!{
                    $callback_fn_path,
                    $($callback_body_self)*
                    ($($call_args,)*
                    callback,)
                }
            }
        });
    };
}

/// This macro generates the callback fn
#[macro_export]
macro_rules! fn_def {
    (
        $(#[$docs:meta])*
        // Block that contains the callback function body
        $body:block,
        // The name of the callback fn
        $name:ident,
        // The parameters of the callback fn
        $($arg:ident: $typ:ty,)*
        // The generics of the callback fn
        ($($gen:tt)*),
        // The return type
        ($($return_type:tt)*),
        // The self declaration
        ($($callback_body_self:tt)*),
    ) => {
        $(#[$docs])*
        pub fn $name $($gen)*(
            $($callback_body_self)*
            $($arg: $typ,)*
        ) -> $($return_type)* {
            $body
        }
    };
}

/// This macro generates the async fn and the callback fn
#[macro_export]
macro_rules! fnify {
    (
        $(#[$docs:meta])*
        $name:ident,
        body: $body:block,
        parameters: ($($arg:ident: $typ:ty,)*),
        async_body: $async_body:block,
        async_parameters: ($(;$async_mutt:ident)? $($async_arg:ident: $async_typ:ty,)*),
        call_args: ($($call_args:ident,)*),
        generics: ($($gen:tt)*),
        async_generics: ($($async_gen:tt)*),
        return_type: ($($return_type:tt)*),
        call_prefix: ($($call_prefix:tt)*),
        callback_body_self: ($($callback_body_self:tt)*),
    ) => {
        $crate::fn_def!(
            $(#[$docs])*
            $body,
            $name,
            $($arg: $typ,)*
            ($($gen)*),
            ($($return_type)*),
            ($($callback_body_self)*),
        );

        #[cfg(feature = "async")]
        $crate::async_fn_def!(
            $(#[$docs])*
            $async_body,
            $name,
            $($call_prefix)*$name,
            ($(;$async_mutt)? $($async_arg: $async_typ,)*)
            ($($call_args,)*)
            ($($async_gen)*),
            ($($return_type)*),
            ($($callback_body_self)*),
        );
    };
}

/// This macro generates a callback based and a async version of the function
#[macro_export]
macro_rules! asyncify {
    (
        $(#[$docs:meta])*
        $name:ident,
        $callback_name:ident: (impl FnMut($callback_type:ty, $($closure_arg_name:ident: $closure_arg:ty,)*) $($bounds:tt)*),
        call_prefix: ($($call_prefix:tt)*),
        generics: ($($gen:tt)*),
        async_generics: ($($async_gen:tt)*),
        parameters: ($($arg:ident: $typ:ty,)*),
        future: $future:ty,
        return_type: ($($return_type:tt)*),
        body: |($($callback_body_self:tt)*)| $body:block,
    ) => {
        $crate::fnify!{
            $(#[$docs])*
            $name,
            body: $body,
            parameters: ($($arg: $typ,)* $callback_name: impl FnMut($($closure_arg,)* $callback_type) $($bounds)*,),
            async_body: {
                Box::new(move |$($closure_arg_name: $closure_arg,)* callback: $callback_type| {
                    let fut = future_fn($($closure_arg_name,)*);
                    $crate::spawn(async move {
                        let res = fut.await;
                        callback(res);
                    })
                })
            },
            async_parameters: ($($arg: $typ,)* ;mut future_fn: $future,),
            call_args: ($($arg,)*),
            generics: ($($gen)*),
            async_generics: ($($async_gen)*),
            return_type: ($($return_type)*),
            call_prefix: ($($call_prefix)*),
            callback_body_self: ($($callback_body_self)*),
        }
    };
    (
        $(#[$docs:meta])*
        $name:ident,
        $callback_name:ident: (impl FnOnce($callback_type:ty) $($bounds:tt)*),
        call_prefix: ($($call_prefix:tt)*),
        generics: ($($gen:tt)*),
        async_generics: ($($async_gen:tt)*),
        parameters: ($($arg:ident: $typ:ty,)*),
        future: $future:ty,
        return_type: ($($return_type:tt)*),
        body: |($($callback_body_self:tt)*)| $body:block,
    ) => {
        $crate::fnify!{
            $(#[$docs])*
            $name,
            body: $body,
            parameters: ($($arg: $typ,)* $callback_name: impl FnOnce($callback_type) $($bounds)*,),
            async_body: {
                Box::new(move |callback: $callback_type| {
                    let fut = future;
                    $crate::spawn(async move {
                        let res = fut.await;
                        callback(res);
                    })
                })
            },
            async_parameters: ($($arg: $typ,)* ;mut future: $future,),
            call_args: ($($arg,)*),
            generics: ($($gen)*),
            async_generics: ($($async_gen)*),
            return_type: ($($return_type)*),
            call_prefix: ($($call_prefix)*),
            callback_body_self: ($($callback_body_self)*),
        }
    };
}

/// Type of the callback function
#[cfg(target_arch = "wasm32")]
pub type CallbackType<T> = Box<dyn FnOnce(T)>;
#[cfg(not(target_arch = "wasm32"))]
pub type CallbackType<T> = Box<dyn FnOnce(T) + Send + Sync>;

#[cfg(not(target_arch = "wasm32"))]
mod sync {
    pub use Send as MaybeSend;
    pub use Sync as MaybeSync;
}
#[cfg(target_arch = "wasm32")]
mod unsync {
    pub trait MaybeSend {}

    impl<T> MaybeSend for T where T: ?Sized {}

    pub trait MaybeSync {}

    impl<T> MaybeSync for T where T: ?Sized {}
}

#[cfg(not(target_arch = "wasm32"))]
pub use sync::*;

#[cfg(target_arch = "wasm32")]
pub use unsync::*;
