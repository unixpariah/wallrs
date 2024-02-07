# WLRS /wɔːlrəs/ 

Wallpaper management library

Supported environments:

- Wayland

Examples:

```rust
use image;
use wlrs::set_from_memory;

fn main() {
  let wallpaper = image::open("wallpaper.jpg").unwrap();
  set_from_memory(wallpaper);
}
```
