use std::{error::Error, sync::mpsc};

use crate::{helpers::resize_image, WallpaperData};
use image::RgbImage;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, EventQueue, QueueHandle,
};

struct Surface {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layers: Vec<LayerSurface>,
}

impl CompositorHandler for Surface {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }
}

impl OutputHandler for Surface {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for Surface {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        _configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
    }
}

impl ShmHandler for Surface {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl Surface {
    fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> Result<Self, Box<dyn Error>> {
        let compositor = CompositorState::bind(globals, qh)?;
        let layer_shell = LayerShell::bind(globals, qh)?;
        let shm = Shm::bind(globals, qh)?;

        let output_state = OutputState::new(globals, qh);

        let layers = output_state
            .outputs()
            .map(|output| {
                let surface = compositor.create_surface(qh);
                let layer = layer_shell.create_layer_surface(
                    qh,
                    surface,
                    Layer::Background,
                    Some("wlrs"),
                    Some(&output),
                );

                layer.set_anchor(Anchor::all());
                layer.set_exclusive_zone(-1);
                layer.set_keyboard_interactivity(KeyboardInteractivity::None);

                layer.commit();

                layer
            })
            .collect();

        Ok(Self {
            registry_state: RegistryState::new(globals),
            output_state,
            shm,
            layers,
        })
    }

    fn draw(
        &mut self,
        image: RgbImage,
        event_queue: &mut EventQueue<Self>,
    ) -> Result<(), Box<dyn Error>> {
        let _ = event_queue.roundtrip(self);
        let output = self
            .output_state
            .outputs()
            .next()
            .ok_or("No outputs found")?;
        let (width, height) = self
            .output_state
            .info(&output)
            .ok_or("Output info not available")?
            .logical_size
            .ok_or("Logical size not found")?;
        let stride = width * 4;

        let mut pool = SlotPool::new(width as usize * height as usize * 4, &self.shm)?;

        if let Ok((buffer, canvas)) =
            pool.create_buffer(width, height, stride, wl_shm::Format::Xrgb8888)
        {
            if let Ok(image) = resize_image(&image, width as u32, height as u32) {
                canvas.copy_from_slice(&image);
            }

            self.layers.iter().for_each(|layer| {
                layer.set_size(width as u32, height as u32);
                layer.wl_surface().damage_buffer(0, 0, width, height);
                layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
                layer.commit();
            });
        }
        Ok(())
    }
}

pub fn wayland(rx: mpsc::Receiver<WallpaperData>) -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;

    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let mut surface = Surface::new(&globals, &qh)?;

    loop {
        event_queue.blocking_dispatch(&mut surface)?;
        if let Ok(wallpaper_data) = rx.recv() {
            let _ = surface.draw(wallpaper_data.image, &mut event_queue);
        }
    }
}

delegate_compositor!(Surface);
delegate_output!(Surface);
delegate_shm!(Surface);

delegate_layer!(Surface);

delegate_registry!(Surface);

impl ProvidesRegistryState for Surface {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
