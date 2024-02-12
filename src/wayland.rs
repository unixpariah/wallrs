use crate::helpers::resize_image;
use image::RgbaImage;
use smithay_client_toolkit::{
    default_environment,
    environment::SimpleGlobal,
    new_default_environment,
    output::with_output_info,
    reexports::{
        calloop,
        client::{
            protocol::{wl_output, wl_shm, wl_surface},
            Attached, Main,
        },
        protocols::wlr::unstable::layer_shell::v1::client::{
            zwlr_layer_shell_v1,
            zwlr_layer_surface_v1::{self, KeyboardInteractivity},
        },
    },
    shm::AutoMemPool,
    WaylandSource,
};
use std::{cell::Cell, error::Error, rc::Rc, sync::mpsc};

default_environment!(Env,
    fields = [
        layer_shell: SimpleGlobal<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    ],
    singles = [
        zwlr_layer_shell_v1::ZwlrLayerShellV1 => layer_shell
    ],
);

#[derive(PartialEq, Copy, Clone)]
enum RenderEvent {
    Redraw,
    Kill,
}

struct Surface {
    surface: wl_surface::WlSurface,
    layer_surface: Main<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pool: AutoMemPool,
    dimensions: (u32, u32),
}

impl Surface {
    fn new(
        output: &wl_output::WlOutput,
        surface: wl_surface::WlSurface,
        layer_shell: &Attached<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
        pool: AutoMemPool,
    ) -> Result<Self, Box<dyn Error>> {
        let (width, height) = with_output_info(output, |info| {
            let dimensions = info.modes[0].dimensions;
            (dimensions.0 as u32, dimensions.1 as u32)
        })
        .ok_or("Coult not get output info")?;
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            Some(output),
            zwlr_layer_shell_v1::Layer::Background,
            "wlrs".to_owned(),
        );
        layer_surface.set_anchor(zwlr_layer_surface_v1::Anchor::all());
        layer_surface.set_exclusive_zone(-1);
        layer_surface.set_margin(0, 0, 0, 0);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer_surface.set_size(width, height);

        let next_render_event = Rc::new(Cell::new(None::<RenderEvent>));
        let next_render_event_handle = Rc::clone(&next_render_event);
        layer_surface.quick_assign(move |layer_surface, event, _| {
            match (event, next_render_event_handle.get()) {
                (zwlr_layer_surface_v1::Event::Closed, _) => {
                    next_render_event_handle.set(Some(RenderEvent::Kill));
                }
                (
                    zwlr_layer_surface_v1::Event::Configure {
                        serial,
                        width: _,
                        height: _,
                    },
                    next,
                ) if next != Some(RenderEvent::Kill) => {
                    layer_surface.ack_configure(serial);
                    next_render_event_handle.set(Some(RenderEvent::Redraw));
                }
                (_, _) => {}
            }
        });

        surface.commit();

        Ok(Self {
            surface,
            layer_surface,
            pool,
            dimensions: (width, height),
        })
    }

    fn draw(&mut self, image: RgbaImage) {
        let stride = 4 * self.dimensions.0 as i32;
        let width = self.dimensions.0 as i32;
        let height = self.dimensions.1 as i32;
        if let Ok((canvas, buffer)) =
            self.pool
                .buffer(width, height, stride, wl_shm::Format::Xrgb8888)
        {
            if let Ok(image) = resize_image(image, width as u32, height as u32) {
                canvas.copy_from_slice(&*image);
            };
            self.surface.attach(Some(&buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, width as i32, height as i32);
            self.surface.commit();
        };
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        self.layer_surface.destroy();
        self.surface.destroy();
    }
}

pub fn wayland(rx: mpsc::Receiver<RgbaImage>) -> Result<(), Box<dyn Error>> {
    let (env, display, queue) =
        new_default_environment!(Env, fields = [layer_shell: SimpleGlobal::new(),])?;

    let surface = env.create_surface().detach();
    let pool = env.create_auto_pool()?;

    let layer_shell = env.require_global::<zwlr_layer_shell_v1::ZwlrLayerShellV1>();
    let output = env
        .get_all_outputs()
        .first()
        .ok_or("Output not found")?
        .to_owned();
    let mut surface_wrapper = Surface::new(&output, surface, &layer_shell.clone(), pool)?;
    let mut event_loop = calloop::EventLoop::<()>::try_new()?;
    WaylandSource::new(queue).quick_insert(event_loop.handle())?;

    loop {
        display.flush()?;
        event_loop.dispatch(None, &mut ())?;
        if let Ok(img) = rx.recv() {
            surface_wrapper.draw(img);
        }
    }
}
