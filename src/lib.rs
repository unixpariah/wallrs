mod wayland;

use image::DynamicImage;
use std::{
    env,
    error::Error,
    path::Path,
    sync::{mpsc, Mutex, Once},
    thread,
};
use wayland::wayland;

static START: Once = Once::new();
static mut SENDER: Mutex<Option<mpsc::Sender<DynamicImage>>> = Mutex::new(None);

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
    T: Into<DynamicImage>,
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
                "wayland" => {
                    wayland(rx).map_err(|_| "Failed to set wallpaper using wayland")?;
                }
                session_type => {
                    return Err(format!("Unsupported session type {}", session_type).into());
                }
            }
            Ok(())
        });
    });

    unsafe {
        if let Some(sender) = SENDER
            .lock()
            .map_err(|_| "Failed to acquire lock")?
            .as_ref()
        {
            sender.send(image.into())?;
        }
    }

    Ok(())
}
