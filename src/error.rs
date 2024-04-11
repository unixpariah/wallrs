use smithay_client_toolkit::{
    reexports,
    shm::{slot::CreateBufferError, CreatePoolError},
};
use std::sync::{mpsc, MutexGuard, PoisonError};
use wayland_client::globals::BindError;
use x11rb::errors::{ConnectError, ConnectionError, ReplyError, ReplyOrIdError};

use crate::{wayland::Wlrs, Channel, WallpaperData};

#[allow(clippy::enum_variant_names)]
pub enum WlrsError {
    ImageError(image::ImageError),
    LockError(&'static str),
    CustomError(&'static str),
    SendError(String),
    ReceiverError(mpsc::RecvError),
    UnsupportedError(String),
    WaylandError(String),
    XorgError(&'static str),
    SizeError(&'static str),
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
        WlrsError::WaylandError("Failed to connect to wayland server".into())
    }
}

impl From<wayland_client::globals::GlobalError> for WlrsError {
    fn from(_err: wayland_client::globals::GlobalError) -> WlrsError {
        WlrsError::WaylandError("Failed to get globals".into())
    }
}

impl From<wayland_client::DispatchError> for WlrsError {
    fn from(_err: wayland_client::DispatchError) -> WlrsError {
        WlrsError::WaylandError("Failed to dispatch events".into())
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
        WlrsError::SendError("Failed to send wallpaper data".to_string())
    }
}

impl From<mpsc::RecvError> for WlrsError {
    fn from(err: mpsc::RecvError) -> WlrsError {
        WlrsError::ReceiverError(err)
    }
}

impl From<fast_image_resize::DifferentTypesOfPixelsError> for WlrsError {
    fn from(err: fast_image_resize::DifferentTypesOfPixelsError) -> WlrsError {
        WlrsError::UnsupportedError(err.to_string())
    }
}

impl From<fast_image_resize::ImageBufferError> for WlrsError {
    fn from(err: fast_image_resize::ImageBufferError) -> WlrsError {
        WlrsError::UnsupportedError(err.to_string())
    }
}

impl From<CreatePoolError> for WlrsError {
    fn from(_err: CreatePoolError) -> WlrsError {
        WlrsError::WaylandError("Failed to create shm pool".into())
    }
}

impl From<CreateBufferError> for WlrsError {
    fn from(_err: CreateBufferError) -> WlrsError {
        WlrsError::WaylandError("Failed to create shm buffer".into())
    }
}

impl From<BindError> for WlrsError {
    fn from(err: BindError) -> WlrsError {
        let err_str = Box::leak(err.to_string().into_boxed_str());
        WlrsError::WaylandError(err_str.into())
    }
}

impl From<reexports::calloop::Error> for WlrsError {
    fn from(err: reexports::calloop::Error) -> WlrsError {
        WlrsError::WaylandError(err.to_string())
    }
}

impl From<reexports::calloop::InsertError<reexports::calloop_wayland_source::WaylandSource<Wlrs>>>
    for WlrsError
{
    fn from(
        err: reexports::calloop::InsertError<
            reexports::calloop_wayland_source::WaylandSource<Wlrs>,
        >,
    ) -> WlrsError {
        WlrsError::WaylandError(err.to_string())
    }
}

impl From<mpsc::SendError<Result<(), WlrsError>>> for WlrsError {
    fn from(err: mpsc::SendError<Result<(), WlrsError>>) -> WlrsError {
        WlrsError::SendError(err.to_string())
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
            WlrsError::SizeError(err) => write!(f, "SizeError: {}", err),
        }
    }
}

impl std::error::Error for WlrsError {}
