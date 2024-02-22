# WLRS /wɔːlrəs/ [![crate](https://img.shields.io/crates/v/wlrs.svg)](https://crates.io/crates/wlrs) [![Build Status](https://github.com/unixpariah/wlrs/actions/workflows/tests.yml/badge.svg)](https://github.com/unixpariah/wlrs/actions/workflows/tests.yml) [![codecov](https://codecov.io/gh/unixpariah/wlrs/graph/badge.svg?token=49LRWZ9D1K)](https://codecov.io/gh/unixpariah/wlrs) [![docs](https://docs.rs/wlrs/badge.svg)](https://docs.rs/wlrs/latest/wlrs/index.html) 

Rust library for managing wallpapers

## Supported environments

- Every Wayland compositor that implements layer-shell and xdg-output
- X11 environments that don't have their own wallpaper management

## Examples:

Setting from memory:
```rust
use wlrs::set_from_memory;

fn main() {
  // Setting to first monitor
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper, Some(0)).unwrap();

  // Setting to all monitors
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper, None).unwrap();
}
```

Setting from file path:

```rust
use wlrs::set_from_path;

fn main() {
  // Setting to first monitor
  set_from_path("wallpaper.jpg", Some(0)).unwrap();
  
  // Setting to all monitors
  set_from_path("wallpaper.jpg", None).unwrap();
}
```
