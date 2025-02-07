# egui_flex changelog

## 0.3.0

- Update egui to 0.31

## 0.2.0

- Update to egui 0.30.0
- Add ways to specify the flex container size, via `Flex::width` and related methods
- Implement `Flex::justify`
- Change `Flex::align_content` default to stretch to match the css flexbox behavior
- Improved textedit flex widget behaviour (once https://github.com/emilk/egui/pull/5275 is merged / released)
- Change the default of Flex::wrap to false
- Change the way the FlexWidget trait works, making it more similar to egui::Widget
- Add lots of tests with egui_kittest
- Remove `FlexAlignContent::Normal` and make `FlexAlignContent::Stretch` the default
- Add `FlexItem::shrink` which allows a single item to shrink
- Add `FlexItem::frame` to customize the frame of a single item
- Add `FlexItem::transform` to set a visual `egui::TSTransform` transform for an item
- Add `FlexItem::frame_builder` to customize the frame of a single item via a closure

## 0.1.1

- Add comment about not all `FlexAlignContent` variants being implemented

## 0.1.0

Initial release
