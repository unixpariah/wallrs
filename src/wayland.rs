use std::{error::Error, sync::mpsc};

use crate::{helpers::resize_image, WallpaperData};
use image::RgbImage;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::{calloop, calloop_wayland_source::WaylandSource},
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
    globals::registry_queue_init,
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

struct SimpleLayer {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    layer: LayerSurface,
    pool: SlotPool,
    width: u32,
    height: u32,
}

impl CompositorHandler for SimpleLayer {
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

impl OutputHandler for SimpleLayer {
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

impl LayerShellHandler for SimpleLayer {
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

impl ShmHandler for SimpleLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl SimpleLayer {
    fn new(
        registry_state: RegistryState,
        output_state: OutputState,
        shm: Shm,
        layer: LayerSurface,
    ) -> Result<Self, Box<dyn Error>> {
        let (width, height) = (1920, 1080);

        let pool = SlotPool::new(width as usize * height as usize * 4, &shm)?;

        layer.set_anchor(Anchor::all());
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(width, height);

        layer.commit();

        Ok(SimpleLayer {
            registry_state,
            output_state,
            shm,
            pool,
            width,
            height,
            layer,
        })
    }

    fn draw(&mut self, qh: &QueueHandle<Self>, image: RgbImage) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;
        if let Ok((buffer, canvas)) = self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Xrgb8888,
        ) {
            if let Ok(image) = resize_image(&image, width, height) {
                canvas.copy_from_slice(&image);
            }

            self.layer
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);
            self.layer
                .wl_surface()
                .frame(qh, self.layer.wl_surface().clone());
            let _ = buffer.attach_to(self.layer.wl_surface());
            self.layer.commit();
        }
    }
}

pub fn wayland(rx: mpsc::Receiver<WallpaperData>) -> Result<(), Box<dyn Error>> {
    let conn = Connection::connect_to_env()?;

    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let surface = compositor.create_surface(&qh);
    let layer =
        layer_shell.create_layer_surface(&qh, surface, Layer::Background, Some("wlrs"), None);

    let mut simple_layer = SimpleLayer::new(
        RegistryState::new(&globals),
        OutputState::new(&globals, &qh),
        shm,
        layer,
    )?;
    let mut event_loop = calloop::EventLoop::<SimpleLayer>::try_new()?;
    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;

    loop {
        event_loop.dispatch(None, &mut simple_layer)?;
        if let Ok(wallpaper_data) = rx.recv() {
            simple_layer.draw(&qh, wallpaper_data.image);
        }
    }
}

delegate_compositor!(SimpleLayer);
delegate_output!(SimpleLayer);
delegate_shm!(SimpleLayer);

delegate_layer!(SimpleLayer);

delegate_registry!(SimpleLayer);

impl ProvidesRegistryState for SimpleLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
