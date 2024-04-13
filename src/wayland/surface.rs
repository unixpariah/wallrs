use crate::{
    error::WlrsError,
    helpers::{crop_image, pad, resize_image},
    CropMode, WallpaperData,
};
use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{wlr_layer::LayerSurface, WaylandSurface},
    shm::{slot::SlotPool, Shm},
};
use wayland_client::{globals::GlobalList, protocol::wl_shm, QueueHandle};

use super::Wlrs;

pub(crate) struct Surface {
    pub(crate) output_info: OutputInfo,
    pub(crate) layer_surface: LayerSurface,
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Surface {
    #[inline]
    pub(crate) fn draw(
        &mut self,
        wallpaper_data: &WallpaperData,
        qh: &QueueHandle<Wlrs>,
        globals: &GlobalList,
    ) -> Result<(), WlrsError> {
        let shm = Shm::bind(globals, qh)?;

        let (width, height) = (self.width, self.height);
        let mut pool = SlotPool::new((width * height * 3) as usize, &shm)?;
        let (buffer, canvas) =
            pool.create_buffer(width, height, width * 3, wl_shm::Format::Bgr888)?;

        let image = &wallpaper_data.image;
        let img = match wallpaper_data.crop_mode {
            CropMode::Fit(color) => resize_image(
                image,
                width as u32,
                height as u32,
                color.unwrap_or([0, 0, 0]),
            )?,
            CropMode::No(color) => pad(
                image,
                width as u32,
                height as u32,
                color.unwrap_or([0, 0, 0]),
            )?,
            CropMode::Crop => crop_image(image, width as u32, height as u32)?,
        };

        canvas.copy_from_slice(&img);

        let layer = &self.layer_surface;
        layer.set_size(width as u32, height as u32);
        layer.wl_surface().damage_buffer(0, 0, width, height);
        layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        layer.commit();

        Ok(())
    }

    pub(crate) fn change_size(&mut self) {
        if let Some((width, height)) = self.output_info.logical_size {
            self.width = width;
            self.height = height;
        }
    }

    pub(crate) fn is_configured(&self) -> bool {
        self.width != 0 && self.height != 0
    }
}
