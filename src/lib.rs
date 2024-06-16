mod error;
mod helpers;
mod wayland;
mod x11;

use crate::error::WlrsError;
use smithay_client_toolkit::reexports::calloop;
use std::{
    env,
    num::NonZeroU32,
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
};
use wayland::wayland;
use x11::x11;

pub(crate) struct WallpaperData {
    image_data: ImageData,
    outputs: Arc<[String]>,
    crop_mode: CropMode,
}

impl WallpaperData {
    fn new(image_data: ImageData, outputs: Arc<[String]>, crop_mode: CropMode) -> Self {
        Self {
            image_data,
            outputs,
            crop_mode,
        }
    }
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

/// Tactic to use when resizing the image
pub enum CropMode {
    /// Center the image and fill the remaining space with specified color, defaults to black
    No(Option<[u8; 3]>),
    /// Resize the image to fit the screen while maintaining aspect ratio, filling the remaining with specified color if provided, defaults to black
    Fit(Option<[u8; 3]>),
    /// Crop the image to fit the screen
    Crop,
}

/// Struct used to represent image
#[derive(Clone)]
pub struct ImageData {
    data: Arc<[u8]>,
    width: NonZeroU32,
    height: NonZeroU32,
}

impl ImageData {
    /// Creates ImageData if pixel format is correct
    pub fn new(data: &[u8], width: NonZeroU32, height: NonZeroU32) -> Option<Self> {
        if data.len() != (width.get() * height.get() * 3) as usize {
            return None;
        }

        Some(Self {
            data: data.into(),
            width,
            height,
        })
    }

    /// # Safety
    ///
    /// Pixel format must be RGB
    pub unsafe fn new_unchecked(data: &[u8], width: NonZeroU32, height: NonZeroU32) -> Self {
        Self {
            data: data.into(),
            width,
            height,
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width.into(), self.height.into())
    }

    pub fn width(&self) -> u32 {
        self.width.into()
    }

    pub fn height(&self) -> u32 {
        self.height.into()
    }
}

static CHANNEL: Mutex<Option<Channel>> = Mutex::new(None);

/// Set the wallpaper from a file path
///
/// # Example
///
/// ```no_run
/// use wlrs::{set_from_path, CropMode};
///
/// // Set to single output
/// set_from_path("path/to/image.png", &["eDP-1".to_string()], CropMode::Fit(None)).unwrap();
///
/// // Set to multiple outputs
/// set_from_path("path/to/image.png", &["eDP-1".to_string(), "HDMI-A-1".to_string()], CropMode::Fit(None)).unwrap();
///
/// // Set to all outputs
/// set_from_path("path/to/image.png", &[], CropMode::Fit(None)).unwrap();
/// ```
pub fn set_from_path<T>(path: T, outputs: &[String], crop_mode: CropMode) -> Result<(), WlrsError>
where
    T: AsRef<Path>,
{
    let image = image::open(path)?.to_rgb8();
    let image_data = ImageData::new(
        image.as_raw(),
        NonZeroU32::new(image.width()).ok_or(WlrsError::SizeError("Image is of width 0"))?,
        NonZeroU32::new(image.height()).ok_or(WlrsError::SizeError("Image is of height 0"))?,
    )
    .unwrap();
    set_from_memory(image_data, outputs, crop_mode)?;
    Ok(())
}

/// Set the wallpaper from a memory
///
/// # Example
///
/// ```
/// use wlrs::{set_from_memory, CropMode, ImageData};
/// use std::num::NonZeroU32;
///
/// // Set to single output
/// let image = ImageData::new(&[0; 1920 * 1080 * 3], NonZeroU32::new(1920).unwrap(), NonZeroU32::new(1080).unwrap()).unwrap();
/// set_from_memory(image, &["eDP-1".to_string()], CropMode::Fit(None)).unwrap();
///
/// // Set to multiple outputs
/// let image = ImageData::new(&[0; 1920 * 1080 * 3], NonZeroU32::new(1920).unwrap(), NonZeroU32::new(1080).unwrap()).unwrap();
/// set_from_memory(image, &["eDP-1".to_string(), "HDMI-A-1".to_string()], CropMode::Fit(None)).unwrap();
///
/// // Set to all outputs
/// let image = ImageData::new(&[0; 1920 * 1080 * 3], NonZeroU32::new(1920).unwrap(), NonZeroU32::new(1080).unwrap()).unwrap();
/// set_from_memory(image, &[], CropMode::Fit(None)).unwrap();
/// ```
pub fn set_from_memory(
    image_data: ImageData,
    outputs: &[String],
    crop_mode: CropMode,
) -> Result<(), WlrsError> {
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

    let wallpaper_data = WallpaperData::new(image_data, outputs.into(), crop_mode);

    channel.as_ref().unwrap().send(wallpaper_data)?;
    match channel.as_ref().unwrap().get_reply() {
        Ok(()) => Ok(()),
        Err(e) => {
            *channel = None;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let img_data = ImageData::new(
            &[0; 1080 * 1920 * 3],
            NonZeroU32::new(1920).unwrap(),
            NonZeroU32::new(1080).unwrap(),
        );
        assert!(img_data.is_some());
        let img_data = img_data.unwrap();
        assert_eq!(
            img_data.data.len(),
            (img_data.width() * img_data.height() * 3) as usize
        );
    }
}
