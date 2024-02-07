# WLRS /wɔːlrəs/ 

Wallpaper management library

Supported environments:

- Wayland

## Examples:

Setting from memory:
```rust
use image;
use wlrs::set_from_memory;

fn main() {
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper);
}
```

Setting from file path:

```rust
use wlrs::set_from_memory;

fn main() {
  set_from_path("wallpaper.jpg");
}
```
