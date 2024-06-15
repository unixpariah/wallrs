use crate::error::WlrsError;
use smithay_client_toolkit::{
    output::OutputInfo,
    shell::{wlr_layer::LayerSurface, WaylandSurface},
    shm::slot::Buffer,
};

pub(crate) struct Surface {
    pub(crate) output_info: OutputInfo,
    pub(crate) layer_surface: LayerSurface,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) canvas: &'static mut [u8],
    pub(crate) buffer: Buffer,
}

impl Surface {
    pub(crate) fn draw(&mut self) -> Result<(), WlrsError> {
        let (width, height) = (self.width, self.height);

        let layer = &self.layer_surface;
        layer.set_size(width as u32, height as u32);
        layer.wl_surface().damage_buffer(0, 0, width, height);
        layer
            .wl_surface()
            .attach(Some(self.buffer.wl_buffer()), 0, 0);
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
