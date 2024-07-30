mod error;
mod helpers;
mod wayland;
mod x11;

use crate::error::WlrsError;
use image::GenericImageView;
use smithay_client_toolkit::reexports::calloop;
use std::{
    env,
    num::NonZeroU32,
    sync::{mpsc, Arc},
    thread,
};
use wayland::wayland;
use x11::x11;

pub(crate) struct WallpaperData {
    image_data: Image,
    outputs: Arc<[String]>,
    crop_mode: CropMode,
}

impl WallpaperData {
    fn new(image_data: Image, outputs: Arc<[String]>, crop_mode: CropMode) -> Self {
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
pub struct Image {
    data: Arc<[u8]>,
    width: NonZeroU32,
    height: NonZeroU32,
}

impl Image {
    /// Creates ImageData if pixel format is correct
    pub fn new(data: &[u8], width: NonZeroU32, height: NonZeroU32) -> Option<Self> {
        if data.len() != (width.get() * height.get() * 4) as usize {
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
    /// Pixel format must be RGBA
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

pub enum SetType<'a> {
    Path(&'a str),
    Img(Image),
}

pub struct Wlrs {
    channel: Channel,
}

impl Wlrs {
    pub fn new() -> Result<Self, WlrsError> {
        let (tx, rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();

        let (ping, ping_source) = calloop::ping::make_ping().unwrap();

        let channel = Channel::new(tx, res_rx, ping);
        thread::spawn(move || {
            let error = match (env::var("WAYLAND_DISPLAY"), env::var("DISPLAY")) {
                (Ok(_), _) => wayland(rx, res_tx.clone(), ping_source),
                (_, Ok(_)) => x11(rx, res_tx.clone()),
                _ => Err(WlrsError::UnsupportedError("No display server".to_string())),
            };
            _ = res_tx.send(error);
        });

        Ok(Self { channel })
    }

    pub fn set(
        &self,
        set_type: SetType,
        outputs: &[String],
        crop_mode: CropMode,
    ) -> Result<(), WlrsError> {
        let image_data = match set_type {
            SetType::Path(path) => {
                let image = image::open(path)?.to_rgba8();
                let width = NonZeroU32::new(image.width())
                    .ok_or(WlrsError::SizeError("Image is of width 0"))?;
                let height = NonZeroU32::new(image.height())
                    .ok_or(WlrsError::SizeError("Image is of height 0"))?;

                Image::new(image.as_raw(), width, height).unwrap()
            }
            SetType::Img(image) => image,
        };
        let wallpaper_data = WallpaperData::new(image_data, outputs.into(), crop_mode);

        self.channel.send(wallpaper_data)?;
        self.channel.get_reply()
    }
}
