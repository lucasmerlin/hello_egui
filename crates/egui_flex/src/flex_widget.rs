use crate::{FlexInstance, FlexItem};

/// Implement this trait for a widget to make it usable in a flex container.
///
/// The reason there is a separate trait is that we need to measure the content size independently
/// of the frame size. (The content will stay at it's intrinsic size while the frame will be
/// stretched according to the flex layout.)
///
/// If your widget has no frmae you don't need to implement this trait and can use
/// [`crate::FlexInstance::add_widget`] to add any [`egui::Widget`].
pub trait FlexWidget {
    /// The response type of the widget
    type Response;

    /// Returns the default [`FlexItem`] for this widget.
    /// Implement this to allow overriding the item config.
    #[must_use] fn default_item() -> FlexItem<'static> {
        FlexItem::new()
    }

    /// Show your widget here. Use the provided [`FlexItem`] to set the flex properties.
    /// Usually you only want to add a single thing to the [`FlexInstance`], as this is what
    /// the user expects.
    fn flex_ui(self, item: FlexItem<'_>, flex_instance: &mut FlexInstance<'_>) -> Self::Response;
}

mod egui_widgets {
    use super::FlexWidget;
    use crate::{FlexInstance, FlexItem};
    use egui::widgets::{
        Button, Checkbox, DragValue, Hyperlink, Image, ImageButton, Label, Link, ProgressBar,
        RadioButton, SelectableLabel, Slider, Spinner, TextEdit,
    };

    macro_rules! impl_widget {
        ($($widget:ty),*) => {
            $(
                impl FlexWidget for $widget {
                    type Response = egui::Response;

                    fn flex_ui(self, item: FlexItem<'_>, instance: &mut FlexInstance<'_>) -> Self::Response {
                        instance.add_widget(item, self).inner
                    }
                }
            )*
        };
    }
    impl_widget!(
        Button<'_>,
        Label,
        Checkbox<'_>,
        Image<'_>,
        DragValue<'_>,
        Hyperlink,
        ImageButton<'_>,
        ProgressBar,
        RadioButton,
        Link,
        SelectableLabel,
        Slider<'_>,
        TextEdit<'_>,
        Spinner
    );
}
