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

pub fn set_from_path<T>(path: T) -> Result<(), Box<dyn Error + Send + Sync>>
where
    T: AsRef<Path>,
{
    let image = image::open(path)?;
    set_from_memory(image)?;
    Ok(())
}

pub fn set_from_memory<T>(image: T) -> Result<(), Box<dyn Error + Send + Sync>>
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
            match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
                "wayland" => wayland(rx).map_err(|_| "Wayland failed")?,
                // When running X11 with startx XDG_SESSION_TYPE is set to tty as its what was used when logging in
                // TODO: Implement x11
                "X11" | "tty" => x11(rx).map_err(|_| "X11 failed")?,
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
