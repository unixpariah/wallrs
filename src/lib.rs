mod helpers;
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

struct WallpaperData {
    image: RgbImage,
    output_num: Vec<u8>,
}

static START: Mutex<Once> = Mutex::new(Once::new());
static SENDER: Mutex<Option<mpsc::Sender<WallpaperData>>> = Mutex::new(None);
static FUTURE: Mutex<Option<mpsc::Receiver<bool>>> = Mutex::new(None);

/// Set the wallpaper from a file path
///
/// # Example
///
/// ```no_run
/// use wlrs::set_from_path;
///
/// // Set to first monitor
/// set_from_path("path/to/image.png", vec![0]).unwrap();
///
/// // Set to multiple monitors
/// set_from_path("path/to/image.png", vec![0, 1]).unwrap();
///
/// // Set to all monitors
/// set_from_path("path/to/image.png", Vec::new()).unwrap();
/// ```
pub fn set_from_path<T>(path: T, output_num: Vec<u8>) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path>,
{
    let image = image::open(path)?;
    set_from_memory(image, output_num)?;
    Ok(())
}

/// Set the wallpaper from a memory
///
/// # Example
///
/// ```rust
/// use image::RgbImage;
/// use wlrs::set_from_memory;
///
/// // Set to first monitor
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, vec![0]).unwrap();
///
/// // Set to multiple monitors
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, vec![0, 1]).unwrap();
///
/// // Set to all monitors
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, Vec::new()).unwrap();
/// ```
pub fn set_from_memory<T>(image: T, output_num: Vec<u8>) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: Into<RgbImage>,
{
    START.lock().unwrap().call_once(|| {
        let (tx, rx) = mpsc::channel();
        if let Ok(mut sender) = SENDER.lock() {
            *sender = Some(tx);
        };

        let (tx, res) = mpsc::channel();
        if let Ok(mut future) = FUTURE.lock() {
            *future = Some(res);
        };

        thread::spawn(move || {
            _ = match env::var("XDG_SESSION_TYPE")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "wayland" => wayland(rx, tx),
                "x11" | "tty" => x11(rx),
                session_type => Err(format!("Unsupported session type: {}", session_type).into()),
            };
        });
    });

    let wallpaper_data = WallpaperData {
        image: image.into(),
        output_num,
    };
    let sender = SENDER.lock().map_err(|_| "Failed to acquire lock")?;
    sender.as_ref().unwrap().send(wallpaper_data)?;

    let future = FUTURE.lock().map_err(|_| "Failed to acquire lock")?;
    match future.as_ref().unwrap().recv() {
        Ok(true) => (),
        Ok(false) => {
            let mut start = START.lock().unwrap();
            *start = Once::new();
            return Err("Failed to set wallpaper".into());
        }
        Err(_) => {
            let mut start = START.lock().unwrap();
            *start = Once::new();
            return Err("Failed to set wallpaper".into());
        }
    };

    Ok(())
}
