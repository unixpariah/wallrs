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
            Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use std::{collections::HashMap, error::Error, sync::mpsc};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

struct OutputDetails {
    output_id: u32,
    layer_surface: LayerSurface,
    output: wl_output::WlOutput,
    configured: bool,
}

struct Surface {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    compositor_state: CompositorState,
    layer_shell: LayerShell,
    outputs: Vec<OutputDetails>,
    cache: HashMap<i32, Vec<u8>>,
}

impl Surface {
    fn new(globals: &GlobalList, qh: &wayland_client::QueueHandle<Self>) -> Self {
        let compositor_state =
            CompositorState::bind(globals, qh).expect("Failed to bind compositor");
        let layer_shell = LayerShell::bind(globals, qh).expect(
            "Failed to bind layer shell, check if the compositor supports layer shell protocol.",
        );
        let shm = Shm::bind(globals, qh).expect("Failed to bind shm");

        Self {
            compositor_state,
            layer_shell,
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            shm,
            outputs: Vec::new(),
            cache: HashMap::new(),
        }
    }

    fn draw(&mut self, image: RgbImage, output_num: Vec<u8>) -> Result<(), Box<dyn Error>> {
        self.outputs
            .iter()
            .enumerate()
            .try_for_each(|(index, output)| {
                if !output_num.contains(&(index as u8)) && !output_num.is_empty() {
                    return Ok::<(), Box<dyn Error>>(());
                }

                let info = self.output_state.info(&output.output).ok_or("")?;
                let (width, height) = info.logical_size.ok_or("")?;
                let mut pool = SlotPool::new((width * height * 4) as usize, &self.shm)?;
                let (buffer, canvas) =
                    pool.create_buffer(width, height, width * 4, wl_shm::Format::Xrgb8888)?;
                if self.cache.get(&width).is_none() {
                    let resized_image = resize_image(&image, width as u32, height as u32)?;
                    self.cache.insert(width, resized_image);
                }

                // This unwrap is safe because we just inserted the value
                let img = self.cache.get(&width).unwrap();
                canvas.copy_from_slice(img);

                let layer = &output.layer_surface;
                layer.set_size(width as u32, height as u32);
                layer.wl_surface().damage_buffer(0, 0, width, height);
                layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
                layer.commit();

                Ok::<(), Box<dyn Error>>(())
            })
    }
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
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let surface = self.compositor_state.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Background,
            Some("ssb"),
            Some(&output),
        );

        if let Some(info) = self.output_state.info(&output) {
            if let Some((width, height)) = info.logical_size {
                layer.set_size(width as u32, height as u32);
                layer.set_anchor(Anchor::all());
                layer.set_exclusive_zone(height);
                layer.commit();

                self.outputs.push(OutputDetails {
                    output_id: info.id,
                    layer_surface: layer,
                    output,
                    configured: false,
                });
            }
        }
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
        output: wl_output::WlOutput,
    ) {
        if let Some(output_info) = self.output_state.info(&output) {
            self.outputs.retain(|info| info.output_id != output_info.id);
        }
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

pub fn wayland(
    rx: mpsc::Receiver<WallpaperData>,
    tx: mpsc::Sender<bool>,
) -> Result<(), Box<dyn Error>> {
    let mut wallpaper_data = rx.recv()?;
    let conn = match Connection::connect_to_env() {
        Ok(conn) => conn,
        Err(_) => {
            _ = tx.send(false);
            return Err("Failed to connect to wayland server".into());
        }
    };

    let (globals, mut event_queue) = match registry_queue_init(&conn) {
        Ok(reg) => reg,
        Err(_) => {
            _ = tx.send(false);
            return Err("Failed to initialize registry".into());
        }
    };
    let qh = event_queue.handle();
    let mut surface = Surface::new(&globals, &qh);
    if event_queue.blocking_dispatch(&mut surface).is_err() {
        _ = tx.send(false);
        return Err("Failed to dispatch event".into());
    };

    loop {
        if surface.outputs.iter().all(|output| output.configured) && !surface.outputs.is_empty() {
            if surface
                .draw(wallpaper_data.image, wallpaper_data.output_num)
                .is_err()
            {
                _ = tx.send(false);
                return Err("".into());
            };

            _ = tx.send(true);
            wallpaper_data = rx.recv()?;
        }

        if event_queue.blocking_dispatch(&mut surface).is_err() {
            _ = tx.send(false);
            return Err("Failed to dispatch event".into());
        }

        surface
            .outputs
            .iter_mut()
            .for_each(|output| output.configured = true);
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
