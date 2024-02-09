# WLRS /wɔːlrəs/ [![crate](https://img.shields.io/crates/v/wlrs.svg)](https://crates.io/crates/wlrs) [![docs](https://docs.rs/wlrs/badge.svg)](https://docs.rs/wlrs/latest/wlrs/index.html)

Rust library for managing wallpapers

## Supported environments:

- Wayland

## Examples:

Setting from memory:
```rust
use image;
use wlrs::set_from_memory;

fn main() {
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper).unwrap();
}
```

Setting from file path:

```rust
use wlrs::set_from_path;

fn main() {
  set_from_path("wallpaper.jpg").unwrap();
}
```
