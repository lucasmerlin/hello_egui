use crate::{FlexContainerResponse, FlexContainerUi};
use egui::Ui;

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
    /// Show your widget here. Use the provided [`Ui`] to draw the container (e.g. using a [`egui::Frame`])
    /// and in the frame ui use [`FlexContainerUi::content`] to draw your widget.
    /// The frame will grow according to the flex layout while the content will be centered / positioned
    /// based on [`crate::FlexItem::align_self_content`].
    fn flex_ui(
        self,
        ui: &mut Ui,
        container: FlexContainerUi,
    ) -> FlexContainerResponse<Self::Response>;
}

mod egui_widgets {
    use super::{FlexContainerResponse, FlexContainerUi, FlexWidget, Ui};
    use egui::widgets::{
        Button, Checkbox, DragValue, Hyperlink, Image, ImageButton, Label, Link, ProgressBar,
        RadioButton, SelectableLabel, Slider, Spinner, TextEdit,
    };
    macro_rules! impl_widget {
        ($($widget:ty),*) => {
            $(
                impl FlexWidget for $widget {
                    type Response = egui::Response;

                    fn flex_ui(self, ui: &mut Ui, container: FlexContainerUi) -> FlexContainerResponse<Self::Response> {
                        container.content_widget(ui, self)
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
