pub mod helpers;
mod wayland;
mod x11;

use image::RgbImage;
use std::{
    env,
    error::Error,
    path::Path,
    sync::{mpsc, Mutex, Once},
    thread,
};
use wayland::wayland;
use x11::x11;

static START: Once = Once::new();
static mut SENDER: Mutex<Option<mpsc::Sender<RgbImage>>> = Mutex::new(None);

/// Set the wallpaper from a file path
///
/// # Example
///
/// ```no_run
/// use wlrs::set_from_path;
///
/// // Set to first monitor
/// set_from_path("path/to/image.png", Some(0)).unwrap();
///
/// // Set to all monitors
/// set_from_path("path/to/image.png", None).unwrap();
/// ```
pub fn set_from_path<T>(path: T, output_num: Option<u8>) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path>,
{
    let image = image::open(path)?;
    set_from_memory(image, output_num)?;
    Ok(())
}

/// Set the wallpaper from a memory buffer
///
/// # Example
///
/// ```rust
/// use image::RgbImage;
/// use wlrs::set_from_memory;
///
/// // Set to first monitor
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, Some(0)).unwrap();
///
/// // Set to all monitors
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, None).unwrap();
/// ```
pub fn set_from_memory<T>(
    image: T,
    output_num: Option<u8>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: Into<RgbImage>,
{
    START.call_once(|| {
        let (tx, rx) = mpsc::channel();
        unsafe {
            if let Ok(mut sender) = SENDER.lock() {
                *sender = Some(tx);
            };
        }
        thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
            match env::var("XDG_SESSION_TYPE")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "wayland" => wayland(rx, output_num).map_err(|_| "Wayland failed")?,
                "x11" | "tty" => x11(rx).map_err(|_| "X11 failed")?,
                session_type => {
                    return Err(format!("Unsupported session type {}", session_type).into())
                }
            }
            Ok(())
        });
    });

    unsafe {
        let sender = SENDER.lock().map_err(|_| "Failed to acquire lock")?;
        sender
            .as_ref()
            .expect("It will always be Some at this point")
            // If this throws it means that rx was dropped due to error in the background thread
            .send(image.into())?;
    }

    Ok(())
}
