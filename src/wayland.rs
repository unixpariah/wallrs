use crate::helpers::resize_image;
use image::RgbImage;
use smithay_client_toolkit::{
    default_environment,
    environment::SimpleGlobal,
    new_default_environment,
    output::with_output_info,
    reexports::{
        calloop,
        client::{
            protocol::{
                wl_output::{self, WlOutput},
                wl_shm, wl_surface,
            },
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
use std::{error::Error, sync::mpsc};

default_environment!(Env,
    fields = [
        layer_shell: SimpleGlobal<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    ],
    singles = [
        zwlr_layer_shell_v1::ZwlrLayerShellV1 => layer_shell
    ],
);

struct Surface {
    output_num: u8,
    surface: wl_surface::WlSurface,
    layer_surface: Main<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pool: AutoMemPool,
    dimensions: (u32, u32),
}

impl Surface {
    fn new(
        output_num: u8,
        output: &wl_output::WlOutput,
        surface: wl_surface::WlSurface,
        layer_shell: &Attached<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
        pool: AutoMemPool,
    ) -> Result<Self, Box<dyn Error>> {
        let (width, height) = with_output_info(output, |info| {
            (
                info.modes[0].dimensions.0 as u32,
                info.modes[0].dimensions.1 as u32,
            )
        })
        .ok_or("Could not get output info")?;

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
        layer_surface.quick_assign(move |layer_surface, event, _| {
            if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
                layer_surface.ack_configure(serial);
            }
        });

        surface.commit();

        Ok(Self {
            output_num,
            surface,
            layer_surface,
            pool,
            dimensions: (width, height),
        })
    }

    fn draw(&mut self, image: &RgbImage) {
        let stride = 4 * self.dimensions.0 as i32;
        let width = self.dimensions.0 as i32;
        let height = self.dimensions.1 as i32;
        if let Ok((canvas, buffer)) =
            self.pool
                .buffer(width, height, stride, wl_shm::Format::Xrgb8888)
        {
            if let Ok(image) = resize_image(image, width as u32, height as u32) {
                canvas.copy_from_slice(&image);
            };

            self.surface.attach(Some(&buffer), 0, 0);
            self.surface.damage_buffer(0, 0, width, height);
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

pub fn wayland(rx: mpsc::Receiver<RgbImage>, output_num: Option<u8>) -> Result<(), Box<dyn Error>> {
    let (env, display, queue) =
        new_default_environment!(Env, fields = [layer_shell: SimpleGlobal::new(),])?;

    let layer_shell = env.require_global::<zwlr_layer_shell_v1::ZwlrLayerShellV1>();
    let all_outputs = env.get_all_outputs();
    let outputs: Vec<(u8, &WlOutput)> = match output_num {
        Some(output_num) => vec![(
            output_num,
            all_outputs
                .get(output_num as usize)
                .ok_or("Output not found")?,
        )],
        None => all_outputs
            .iter()
            .enumerate()
            .map(|(index, output)| (index as u8, output))
            .collect(),
    };

    let mut surfaces: Vec<Result<Surface, Box<dyn Error>>> = outputs
        .iter()
        .map(|output_info| {
            let surface = env.create_surface().detach();
            let pool = env.create_auto_pool()?;
            Surface::new(output_info.0, output_info.1, surface, &layer_shell, pool)
        })
        .collect();
    let mut event_loop = calloop::EventLoop::<()>::try_new()?;
    WaylandSource::new(queue).quick_insert(event_loop.handle())?;

    loop {
        display.flush()?;
        event_loop.dispatch(None, &mut ())?;
        if let Ok(img) = rx.recv() {
            surfaces
                .iter_mut()
                .enumerate()
                .for_each(|(index, surface)| {
                    if let Ok(surface) = surface {
                        if surface.output_num == index as u8 || output_num.is_none() {
                            surface.draw(&img);
                        }
                    }
                });
        }
    }
}
