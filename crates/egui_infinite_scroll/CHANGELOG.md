# egui_infinite_scroll changelog

## 0.7.0

- Update egui to 0.31

## 0.6.0

- update egui to 0.30

## 0.5.0

- update egui to 0.29

## 0.4.0

- update egui to 0.28

## 0.3.0

- update egui to 0.27
- **breaking**: if you use a start_loader, with egui 0.27
  you will need to disable animations for the ScrollArea, using `ScrollArea::animated`.

  This can be reverted once egui 0.28 is released, which will hopefully include a better
  way to specify scroll animations

## 0.2.1

- expose the virtual_list field of the InfiniteScroll struct, to allow customizing virtual list settings
- fix loading() returning true when the list is completely empty

## 0.2.0

- update egui to 0.26

## 0.1.0

Initial release
