#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::{Align, Label, Layout, Ui, Vec2, WidgetText};
use std::future::Future;

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

#[cfg(feature = "tokio")]
pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    tokio::task::spawn(future);
}

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

struct T;

impl T {
    fn test() {}
}

/// This macro generates the async fn
#[macro_export]
macro_rules! async_fn_def {
    (
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
            $body,
            $name,
            $($arg: $typ,)*
            ($($gen)*),
            ($($return_type)*),
            ($($callback_body_self)*),
        );

        $crate::async_fn_def!(
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

#[macro_export]
macro_rules! asyncify {
    (
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

pub type CallbackType<T> = Box<dyn FnOnce(T) + Send>;

struct Test {}

impl Test {
    // asyncify! {
    //     call_just_self,
    //     FnMut,
    //     callback_mut: (impl FnMut(CallbackType<Result<(), ()>>,)),
    //     call_prefix: (Self::),
    //     self_usage: (mut_self),
    //     generics: (),
    //     async_generics: (<F: Future<Output = Result<(), ()>> + Send + 'static>),
    //     parameters: (),
    //     future: (impl FnMut() -> F),
    //     return_type: (Self),
    //     body: |(self,)| {
    //         println!("Hello world!");
    //         Self {}
    //     },
    // }
    // asyncify! {
    //     call_ref_mut_self,
    //     callback_mut: CallbackType<Result<(), ()>>,
    //     future: impl FnMut() -> F,
    //     body: {
    //         println!("Hello world!");
    //         "Hello world!".to_string()
    //     },
    //     parameters: (),
    //     call_prefix: (Self::),
    //     generics: (),
    //     async_generics: (<F: Future<Output = Result<(), ()>> + Send + 'static>),
    //     return_type: (String),
    //     self_usage: (ref_mut_self),
    // }
    //
    // asyncify! {
    //     call_ref_self,
    //     callback_mut: CallbackType<Result<(), ()>>,
    //     future: impl FnMut() -> F,
    //     body: {
    //         println!("Hello world!");
    //     },
    //     parameters: (),
    //     call_prefix: (Self::),
    //     generics: (<>),
    //     async_generics: (<F: Future<Output = Result<(), ()>> + Send + 'static>),
    //     return_type: (()),
    //     self_usage: (ref_self),
    // }
    //
    //
    // asyncify! {
    //     call_no_self,
    //     callback_mut: CallbackType<Result<(), ()>>,
    //     future: impl FnMut() -> F,
    //     body: {
    //         println!("Hello world!");
    //     },
    //     parameters: (),
    //     call_prefix: (Self::),
    //     generics: (<>),
    //     async_generics: (<F: Future<Output = Result<(), ()>> + Send + 'static>),
    //     return_type: (()),
    //     self_usage: (no_self),
    // }
}

// asyncify! {
//     test_fn,
//     callback_mut: CallbackType<Result<(), ()>>,
//     future: impl FnMut() -> F,
//     body: {
//         println!("Hello world!");
//     },
//     parameters: (),
//     call_prefix: (),
//     generics: (<>),
//     async_generics: (<F: Future<Output = Result<(), ()>> + Send + 'static>),
//     return_type: (()),
//     self_usage: (no_self),
// }

struct Test2 {}

impl Test2 {
    fn call_ref_mut_self(&mut self, callback: impl FnMut(CallbackType<Result<(), ()>>)) {
        {
            println!("Hello world!");
        };
    }
    async fn call_ref_mut_self_asc<F: Future<Output = Result<(), ()>> + Send + 'static>(
        &mut self,
        mut future: impl FnMut() -> F,
    ) {
        let callback = {
            Box::new(|callback: CallbackType<Result<(), ()>>| {
                let fut = future();
                crate::spawn(async move {
                    let res = fut.await;
                    callback(res);
                })
            })
        };

        Self::call_ref_mut_self(self, callback)
    }
}
