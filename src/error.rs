use std::sync::{mpsc, MutexGuard, PoisonError};
use x11rb::errors::{ConnectError, ConnectionError, ReplyError, ReplyOrIdError};

use crate::{Channel, WallpaperData};

#[allow(clippy::enum_variant_names)]
pub enum WlrsError {
    ImageError(image::ImageError),
    LockError(&'static str),
    CustomError(&'static str),
    SendError(&'static str),
    ReceiverError(mpsc::RecvError),
    UnsupportedError(String),
    WaylandError(&'static str),
    XorgError(&'static str),
}

impl std::fmt::Debug for WlrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "WlrsError")
    }
}

impl From<ConnectionError> for WlrsError {
    fn from(_err: ConnectionError) -> WlrsError {
        WlrsError::XorgError("Failed to connect to xorg server")
    }
}

impl From<ReplyError> for WlrsError {
    fn from(_err: ReplyError) -> WlrsError {
        WlrsError::XorgError("Failed to get reply from xorg server")
    }
}

impl From<ReplyOrIdError> for WlrsError {
    fn from(_err: ReplyOrIdError) -> WlrsError {
        WlrsError::XorgError("Failed to get reply or id from xorg server")
    }
}

impl From<ConnectError> for WlrsError {
    fn from(_err: ConnectError) -> WlrsError {
        WlrsError::XorgError("Failed to connect to xorg server")
    }
}

impl From<wayland_client::ConnectError> for WlrsError {
    fn from(_err: wayland_client::ConnectError) -> WlrsError {
        WlrsError::WaylandError("Failed to connect to wayland server")
    }
}

impl From<wayland_client::globals::GlobalError> for WlrsError {
    fn from(_err: wayland_client::globals::GlobalError) -> WlrsError {
        WlrsError::WaylandError("Failed to get globals")
    }
}

impl From<wayland_client::DispatchError> for WlrsError {
    fn from(_err: wayland_client::DispatchError) -> WlrsError {
        WlrsError::WaylandError("Failed to dispatch events")
    }
}

impl From<PoisonError<MutexGuard<'_, Option<Channel>>>> for WlrsError {
    fn from(_err: PoisonError<MutexGuard<Option<Channel>>>) -> WlrsError {
        WlrsError::LockError("Failed to lock channel mutex")
    }
}

impl From<image::ImageError> for WlrsError {
    fn from(err: image::ImageError) -> WlrsError {
        WlrsError::ImageError(err)
    }
}

impl From<mpsc::SendError<WallpaperData>> for WlrsError {
    fn from(_err: mpsc::SendError<WallpaperData>) -> WlrsError {
        WlrsError::SendError("Failed to send wallpaper data")
    }
}

impl From<mpsc::RecvError> for WlrsError {
    fn from(err: mpsc::RecvError) -> WlrsError {
        WlrsError::ReceiverError(err)
    }
}

impl std::fmt::Display for WlrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WlrsError::ImageError(err) => write!(f, "ImageError: {}", err),
            WlrsError::LockError(err) => write!(f, "LockError: {}", err),
            WlrsError::CustomError(err) => write!(f, "CustomError: {}", err),
            WlrsError::SendError(err) => write!(f, "SendError: {}", err),
            WlrsError::ReceiverError(err) => write!(f, "ReceiverError: {}", err),
            WlrsError::UnsupportedError(err) => write!(f, "UnsupportedError: {}", err),
            WlrsError::WaylandError(err) => write!(f, "WaylandError: {}", err),
            WlrsError::XorgError(err) => write!(f, "XorgError: {}", err),
        }
    }
}

impl std::error::Error for WlrsError {}
