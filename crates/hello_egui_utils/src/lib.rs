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
macro_rules! strip_self {
    (mut $self:expr,) => {
        $self
    };
    (&mut $self:expr,) => {
        $self
    };
    (&$self:expr,) => {
        $self
    };
    ($self:expr,) => {
        $self
    };
    () => {};
    ($($tt:tt)*) => {
        $crate::strip_self!($($tt)*,)
    };
}

struct T;

impl T {
    fn test(self) {
        strip_self!(&mut self);
    }
}

/// This macro generates another macro that in turn generates the async fn, based on the self usage
macro_rules! gen_async_fn_def {
    (
        // We need to pass the dollar sign to the macro to `escape` it
        dollar($dollar:tt);
        $((
            $self_type:ident: ($($self:tt)*);
            self_ref: $($self_ref:tt)*
        ),)*
    ) => {

        #[macro_export]
        macro_rules! async_fn_def {
            $(
                (
                    // The type of self that is used in the async fn
                    $self_type,
                    // Block that contains the async logic
                    $dollar body:block,
                    // The name of the async fn
                    $dollar name:ident,
                    // The path to the callback fn (e.g. Self::callback)
                    $dollar callback_fn_path:path,
                    // The parameters of the async fn. The semicolon in front of the mutt is a hack to circumvent ambiguity
                    ($dollar ($dollar (;$dollar mutt:ident)? $dollar arg:ident: $dollar typ:ty,)*)
                    // The parameters of the call to the callback fn
                    ($dollar ($dollar call_args:ident,)*)
                    // The generics of the async fn
                    ($dollar ($dollar gen:tt)*),
                    // Return type
                    ($dollar ($dollar return_type:tt)*),
                    // Self reference
                    ($dollar ($dollar callback_body_self:tt)*),
                ) => {
                    // We use concat_idents to generate the name for the async fn
                    $crate::concat_idents!(fn_name = $dollar name, _async {
                        pub fn fn_name$dollar ($dollar gen)*(
                            $dollar ($dollar callback_body_self)*
                            $dollar ($dollar ($dollar mutt)? $dollar arg: $dollar typ,)*
                        ) -> $dollar ($dollar return_type)* {
                            let callback = $dollar body;

                            // Construct the call to the callback fn
                            $dollar callback_fn_path (
                                $crate::strip_self!($dollar ($dollar callback_body_self)*),
                                $dollar ($dollar call_args,)*
                                callback,
                            )
                        }
                    });
                };
            )*
        }
    };
}

gen_async_fn_def!(
    dollar($);
    (ref_mut_self: (&mut self,); self_ref: self,),
    (ref_self: (&self,); self_ref: self,),
    (mut_self: (self,); self_ref: self,),
    (just_self: (self,); self_ref: self,),
    (no_self: (); self_ref: ),
);

/// This macro generates the callback fn, based on the self usage
macro_rules! gen_fn_def {
    (
        // We need to pass the dollar sign to the macro to `escape` it
        dollar($dollar:tt);
        $((
            $self_type:ident: ($($self:tt)*);
        ),)*
    ) => {

        #[macro_export]
        macro_rules! fn_def {
            $(
                (
                    // The type of self that is used in the async fn
                    $self_type,
                    // Block that contains the callback function body
                    $dollar body:block,
                    // The name of the callback fn
                    $dollar name:ident,
                    // The parameters of the callback fn
                    $dollar ($dollar arg:ident: $dollar typ:ty,)*
                    // The generics of the callback fn
                    ($dollar ($dollar gen:tt)*),
                    // The return type
                    ($dollar ($dollar return_type:tt)*),
                    // The self declaration
                    ($dollar ($dollar callback_body_self:tt)*),
                ) => {
                    pub fn $dollar name $dollar ($dollar gen)*(
                        $dollar ($dollar callback_body_self)*
                        $dollar ($dollar arg: $dollar typ,)*
                    ) -> $dollar ($dollar return_type)* {
                        $dollar body
                    }
                };
            )*
        }
    };
}

gen_fn_def!(
    dollar($);
    (ref_mut_self: (&mut self,);),
    (ref_self: (&self,);),
    (mut_self: (mut self,);),
    (just_self: (self,);),
    (no_self: ();),
);

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
        self_usage: ($self_usage:ident),
        callback_body_self: ($($callback_body_self:tt)*),
    ) => {
        $crate::fn_def!(
            $self_usage,
            $body,
            $name,
            $($arg: $typ,)*
            ($($gen)*),
            ($($return_type)*),
            ($($callback_body_self)*),
        );

        $crate::async_fn_def!(
            $self_usage,
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
        FnMut,
        $callback_name:ident: (impl FnMut($callback_type:ty, $($closure_arg_name:ident: $closure_arg:ty,)*) $($bounds:tt)*),
        call_prefix: ($($call_prefix:tt)*),
        self_usage: ($self_usage:ident),
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
            self_usage: ($self_usage),
            callback_body_self: ($($callback_body_self)*),
        }
    };
    (
        $name:ident,
        FnOnce,
        $callback_name:ident: $callback_type:ty,
        call_prefix: ($($call_prefix:tt)*),
        self_usage: ($self_usage:ident),
        generics: ($($gen:tt)*),
        async_generics: ($($async_gen:tt)*),
        parameters: ($($arg:ident: $typ:ty,)*),
        future: $future:ty,
        return_type: ($($return_type:tt)*),
        body: $body:block,
    ) => {
        $crate::fnify!{
            $name,
            body: $body,
            parameters: ($($arg: $typ,)* $callback_name: impl FnOnce($callback_type) + Send + Sync + 'static,),
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
            self_usage: ($self_usage),
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
