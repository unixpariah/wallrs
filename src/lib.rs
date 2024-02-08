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

pub fn set_from_path<T>(path: T) -> Result<(), Box<dyn Error>>
where
    T: AsRef<Path>,
{
    let image = image::open(path)?;
    set_from_memory(image)?;
    Ok(())
}

pub fn set_from_memory<T>(image: T) -> Result<(), Box<dyn Error>>
where
    T: Into<DynamicImage>,
{
    START.call_once(|| {
        let (tx, rx) = mpsc::channel();
        unsafe {
            let mut sender = SENDER.lock().unwrap();
            *sender = Some(tx);
        }
        let _ = thread::spawn(move || -> Result<(), std::io::Error> {
            match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
                "wayland" => {
                    let _ = wayland(rx);
                }
                session_type => {
                    std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        format!("Unsupported session type {session_type}"),
                    );
                }
            }
            Ok(())
        })
        .join();
    });

    unsafe {
        if let Some(sender) = SENDER.lock()?.as_ref() {
            sender.send(image.into())?;
        }
    }

    Ok(())
}
