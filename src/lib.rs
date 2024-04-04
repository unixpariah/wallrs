mod error;
mod helpers;
mod wayland;
mod x11;

use crate::error::WlrsError;
pub use image::RgbImage;
use std::{
    env,
    path::Path,
    sync::{mpsc, Mutex},
    thread,
};
use wayland::wayland;
use x11::x11;

pub struct WallpaperData {
    image: RgbImage,
    output_num: Vec<u8>,
}

struct Channel {
    sender: mpsc::Sender<WallpaperData>,
    receiver: mpsc::Receiver<bool>,
}

impl Channel {
    fn new(sender: mpsc::Sender<WallpaperData>, receiver: mpsc::Receiver<bool>) -> Self {
        Self { sender, receiver }
    }

    fn send(&self, data: WallpaperData) -> Result<(), mpsc::SendError<WallpaperData>> {
        self.sender.send(data)
    }

    fn recv(&self) -> Result<bool, mpsc::RecvError> {
        self.receiver.recv()
    }
}

static CHANNEL: Mutex<Option<Channel>> = Mutex::new(None);

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
pub fn set_from_path<T>(path: T, output_num: Vec<u8>) -> Result<(), WlrsError>
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
/// ```
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
pub fn set_from_memory<T>(image: T, output_num: Vec<u8>) -> Result<(), WlrsError>
where
    T: Into<RgbImage>,
{
    let mut channel = CHANNEL
        .lock()
        .map_err(|_| WlrsError::LockError("Failed to lock sender"))?;

    if channel.is_none() {
        let (tx, rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();
        *channel = Some(Channel::new(tx, res_rx));

        thread::spawn(move || {
            let _err = match env::var("XDG_SESSION_TYPE")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "wayland" => wayland(rx, res_tx.clone()),
                "x11" | "tty" => x11(rx, res_tx.clone()),
                session_type => Err(format!("Unsupported session type: {}", session_type).into()),
            };
            _ = res_tx.send(false);
        });
    }

    let wallpaper_data = WallpaperData {
        image: image.into(),
        output_num,
    };

    channel.as_ref().unwrap().send(wallpaper_data)?;
    match channel.as_ref().unwrap().recv() {
        Ok(true) => Ok(()),
        Ok(false) => {
            *channel = None;
            Err(WlrsError::CustomError("Failed to set wallpaper"))
        }
        Err(_) => {
            *channel = None;
            Err(WlrsError::CustomError("Failed to set wallpaper"))
        }
    }
}
