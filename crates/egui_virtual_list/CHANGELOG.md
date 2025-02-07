# egui_virtual_list changelog

## 0.7.0

- Update egui to 0.31

## 0.6.0

- Updated egui to 0.30

## 0.5.0

- Updated egui to 0.29

## 0.4.0

- round `last_width` to prevent flickering due to rounding errors when shown in a moving container
- Updated egui to 0.28

## 0.3.0

- Updated egui to 0.27
- **breaking**: if you use a items_inserted_at_start, with egui 0.27
  you will need to disable animations for the ScrollArea, using `ScrollArea::animated`
  (only available with egui 0.27.2).

  This can be reverted once egui 0.28 is released, which will hopefully include a better
  way to specify scroll animations

## 0.2.0

- Updated egui to 0.26

## 0.1.0

Initial release
