use crate::{
    error::WlrsError,
    helpers::{crop_image, pad, resize_image},
    CropMode, WallpaperData,
};
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
use std::sync::mpsc;
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_shm, wl_surface},
    Connection, QueueHandle,
};

struct Surface {
    layer_surface: LayerSurface,
    width: u32,
    height: u32,
    id: u32,
}

impl Surface {
    fn draw(
        &mut self,
        wallpaper_data: &mut WallpaperData,
        qh: &QueueHandle<Wlrs>,
        globals: &GlobalList,
    ) -> Result<(), WlrsError> {
        let shm = Shm::bind(globals, qh)?;

        let (width, height) = (self.width, self.height);
        let mut pool = SlotPool::new((width * height * 3) as usize, &shm)?;
        let (buffer, canvas) = pool.create_buffer(
            width as i32,
            height as i32,
            width as i32 * 3,
            wl_shm::Format::Bgr888,
        )?;

        let image = &mut wallpaper_data.image;
        let img = match wallpaper_data.crop_mode {
            CropMode::Fit(color) => resize_image(image, width, height, color.unwrap_or([0, 0, 0]))?,
            CropMode::No(color) => pad(image, width, height, color.unwrap_or([0, 0, 0]))?,
            CropMode::Crop => crop_image(image, width, height)?,
        };

        canvas.copy_from_slice(&img);

        let layer = &self.layer_surface;
        layer.set_size(width, height);
        layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        layer.wl_surface().attach(Some(buffer.wl_buffer()), 0, 0);
        layer.commit();

        Ok(())
    }

    pub fn change_size(&mut self, configure: LayerSurfaceConfigure, _qh: &QueueHandle<Wlrs>) {
        let (width, height) = configure.new_size;
        self.width = width;
        self.height = height;
    }

    pub fn is_configured(&self) -> bool {
        self.width != 0 && self.height != 0
    }
}

struct Wlrs {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor_state: CompositorState,
    layer_shell: LayerShell,
    surfaces: Vec<Surface>,
    shm: Shm,
}

impl Wlrs {
    fn new(
        globals: &GlobalList,
        qh: &wayland_client::QueueHandle<Self>,
    ) -> Result<Self, WlrsError> {
        let compositor_state = CompositorState::bind(globals, qh)?;
        let layer_shell = LayerShell::bind(globals, qh)?;
        let shm = Shm::bind(globals, qh)?;

        Ok(Self {
            compositor_state,
            layer_shell,
            output_state: OutputState::new(globals, qh),
            registry_state: RegistryState::new(globals),
            surfaces: Vec::new(),
            shm,
        })
    }
}

impl CompositorHandler for Wlrs {
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

impl OutputHandler for Wlrs {
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

        layer.set_anchor(Anchor::all());
        layer.commit();

        self.surfaces.push(Surface {
            layer_surface: layer,
            width: 0,
            height: 0,
            id: 0,
        });
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
            self.surfaces.retain(|info| info.id != output_info.id);
        }
    }
}

impl LayerShellHandler for Wlrs {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        self.surfaces
            .iter_mut()
            .find(|surface| &surface.layer_surface == layer)
            .unwrap()
            .change_size(configure, qh);
    }
}

impl ShmHandler for Wlrs {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

pub fn wayland(
    rx: mpsc::Receiver<WallpaperData>,
    tx: mpsc::Sender<Result<(), WlrsError>>,
) -> Result<(), WlrsError> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();
    let mut wlrs = Wlrs::new(&globals, &qh)?;

    let mut wallpaper_data = rx.recv()?;
    loop {
        let drawn = wlrs
            .surfaces
            .iter_mut()
            .enumerate()
            .map(|(index, surface)| {
                if surface.is_configured()
                    && (wallpaper_data.output_num.contains(&(index as u8))
                        || wallpaper_data.output_num.is_empty())
                {
                    surface.draw(&mut wallpaper_data, &qh, &globals)?;
                    return Ok::<bool, WlrsError>(true);
                }
                Ok(false)
            })
            .reduce(|a: Result<bool, _>, b: Result<bool, _>| match (a, b) {
                (Ok(a), Ok(b)) => Ok(a || b),
                (Err(e), _) | (_, Err(e)) => Err(e),
            })
            .unwrap_or(Ok(false))?;

        if let Err(e) = event_queue.blocking_dispatch(&mut wlrs) {
            return Err(WlrsError::WaylandError(e.to_string()));
        }

        if drawn {
            _ = tx.send(Ok(()));
            wallpaper_data = rx.recv()?;
        }
    }
}

delegate_compositor!(Wlrs);
delegate_output!(Wlrs);
delegate_shm!(Wlrs);
delegate_layer!(Wlrs);
delegate_registry!(Wlrs);

impl ProvidesRegistryState for Wlrs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}
