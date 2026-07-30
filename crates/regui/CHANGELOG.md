# regui changelog

## 0.1.0

- Initial release
- `Regui`: run part of a ui in its own egui viewport and paint it into the parent, scaled,
  rotated or offset
- `Regui::offscreen` (`wgpu` feature): render the child through a texture, for exact
  clipping when rotated and crisp text at any scale
- `Regui::blur` (`wgpu` feature): blur the child's own content
- `BackdropBlur` (`wgpu` feature): blur whatever is already drawn behind a rect, underneath
  a window's own frame
