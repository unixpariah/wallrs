# WLRS /wɔːlrəs/ [![crate](https://img.shields.io/crates/v/wlrs.svg)](https://crates.io/crates/wlrs) [![Build Status](https://github.com/unixpariah/wlrs/actions/workflows/tests.yml/badge.svg)](https://github.com/unixpariah/wlrs/actions/workflows/tests.yml) [![codecov](https://codecov.io/gh/unixpariah/wlrs/graph/badge.svg?token=49LRWZ9D1K)](https://codecov.io/gh/unixpariah/wlrs) [![docs](https://docs.rs/wlrs/badge.svg)](https://docs.rs/wlrs/latest/wlrs/index.html)

Rust library for managing wallpapers

## Supported environments

- Every Wayland compositor that implements layer-shell (e.g. Hyprland, Sway, KDE, etc.)
- X11 environments that don't have their own wallpaper management (e.g. dwm, i3, bspwm, etc.)

## Examples:

Setting from memory:
```rust
use wlrs::set_from_memory;

fn main() {
  // Set to single output
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper, vec!["eDP-1".to_string(), CropMode::Fit(None)]).unwrap();

  // Set to multiple outputs
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper, vec!["eDP-1".to_string(), "HDMI-A-1".to_string(), CropMode::Fit(None)]).unwrap();

  // Set to all outputs
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper, Vec::new(), CropMode::Fit(None)).unwrap();
}
```

Setting from file path:

```rust
use wlrs::set_from_path;

fn main() {
  // Set to single output
  set_from_path("wallpaper.jpg", vec!["eDP-1".to_string(), CropMode::Fit(None)]).unwrap();

  // Set to multiple monitor
  set_from_path("wallpaper.jpg", vec!["eDP-1".to_string(), "HDMI-A-1".to_string(), CropMode::Fit(None)]).unwrap();

  // Set to all outputs
  set_from_path("wallpaper.jpg", Vec::new(), CropMode::Fit(None))unwrap();
}
```
