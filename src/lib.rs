mod error;
mod helpers;
mod wayland;
mod x11;

use crate::error::WlrsError;
pub use image;
use smithay_client_toolkit::reexports::calloop;
use std::{
    env,
    path::Path,
    sync::{mpsc, Mutex},
    thread,
};
use wayland::wayland;
use x11::x11;

#[derive(Clone)]
pub(crate) struct WallpaperData {
    image: image::RgbImage,
    outputs: Vec<usize>,
    crop_mode: CropMode,
}

struct Channel {
    sender: mpsc::Sender<WallpaperData>,
    ping: calloop::ping::Ping,
    receiver: mpsc::Receiver<Result<(), WlrsError>>,
}

impl Channel {
    fn new(
        sender: mpsc::Sender<WallpaperData>,
        receiver: mpsc::Receiver<Result<(), WlrsError>>,
        ping: calloop::ping::Ping,
    ) -> Self {
        Self {
            sender,
            receiver,
            ping,
        }
    }

    fn send(&self, data: WallpaperData) -> Result<(), mpsc::SendError<WallpaperData>> {
        self.sender.send(data)
    }

    fn get_reply(&self) -> Result<(), WlrsError> {
        self.ping.ping();
        self.receiver.recv()?
    }
}

#[derive(Clone)]
/// Tactic to use when resizing the image
pub enum CropMode {
    /// Center the image and fill the remaining space with specified color, defaults to black
    No(Option<[u8; 3]>),
    /// Resize the image to fit the screen while maintaining aspect ratio, filling the remaining with specified color if provided, defaults to black
    Fit(Option<[u8; 3]>),
    /// Crop the image to fit the screen
    Crop,
}

static CHANNEL: Mutex<Option<Channel>> = Mutex::new(None);

/// Set the wallpaper from a file path
///
/// # Example
///
/// ```no_run
/// use wlrs::{set_from_path, CropMode};
///
/// // Set to first monitor
/// set_from_path("path/to/image.png", vec![0], CropMode::Fit(None)).unwrap();
///
/// // Set to multiple monitors
/// set_from_path("path/to/image.png", vec![0, 1], CropMode::Fit(None)).unwrap();
///
/// // Set to all monitors
/// set_from_path("path/to/image.png", Vec::new(), CropMode::Fit(None)).unwrap();
/// ```

pub fn set_from_path<T>(path: T, outputs: Vec<usize>, crop_mode: CropMode) -> Result<(), WlrsError>
where
    T: AsRef<Path>,
{
    let image = image::open(path)?;
    set_from_memory(image, outputs, crop_mode)?;
    Ok(())
}

/// Set the wallpaper from a memory
///
/// # Example
///
/// ```
/// use image::RgbImage;
/// use wlrs::{set_from_memory, CropMode};
///
/// // Set to first monitor
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, vec![0], CropMode::Fit(None)).unwrap();
///
/// // Set to multiple monitors
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, vec![0, 1], CropMode::Fit(None)).unwrap();
///
/// // Set to all monitors
/// let image = RgbImage::new(1920, 1080);
/// set_from_memory(image, Vec::new(), CropMode::Fit(None)).unwrap();
/// ```
pub fn set_from_memory<T>(
    image: T,
    outputs: Vec<usize>,
    crop_mode: CropMode,
) -> Result<(), WlrsError>
where
    T: Into<image::RgbImage>,
{
    let image = image.into();
    if image.width() == 0 || image.height() == 0 {
        return Err(WlrsError::SizeError("Empty image"));
    }

    let mut channel = CHANNEL.lock()?;
    if channel.is_none() {
        let (tx, rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();

        let (ping, ping_source) = calloop::ping::make_ping().unwrap();

        *channel = Some(Channel::new(tx, res_rx, ping));
        thread::spawn(move || {
            let error = match (env::var("WAYLAND_DISPLAY"), env::var("DISPLAY")) {
                (Ok(_), _) => wayland(rx, res_tx.clone(), ping_source),
                (_, Ok(_)) => x11(rx, res_tx.clone()),
                _ => Err(WlrsError::UnsupportedError("No display server".to_string())),
            };
            _ = res_tx.send(error);
        });
    }

    let wallpaper_data = WallpaperData {
        image,
        outputs,
        crop_mode,
    };

    // This is always Some at this point
    channel.as_ref().unwrap().send(wallpaper_data)?;
    match channel.as_ref().unwrap().get_reply() {
        Ok(()) => Ok(()),
        Err(e) => {
            *channel = None;
            Err(e)
        }
    }
}
