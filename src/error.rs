use std::sync::mpsc;

use crate::WallpaperData;

#[allow(clippy::enum_variant_names)]
pub enum WlrsError {
    ImageError(image::ImageError),
    LockError(&'static str),
    CustomError(&'static str),
    SendError(mpsc::SendError<WallpaperData>),
}

impl std::fmt::Debug for WlrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WlrsError")
    }
}

impl From<image::ImageError> for WlrsError {
    fn from(err: image::ImageError) -> WlrsError {
        WlrsError::ImageError(err)
    }
}

impl From<mpsc::SendError<WallpaperData>> for WlrsError {
    fn from(err: mpsc::SendError<WallpaperData>) -> WlrsError {
        WlrsError::SendError(err)
    }
}

impl std::fmt::Display for WlrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WlrsError::ImageError(err) => write!(f, "ImageError: {}", err),
            WlrsError::LockError(err) => write!(f, "LockError: {}", err),
            WlrsError::CustomError(err) => write!(f, "CustomError: {}", err),
            WlrsError::SendError(err) => write!(f, "SendError: {}", err),
        }
    }
}

impl std::error::Error for WlrsError {}
