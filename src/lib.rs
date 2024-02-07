use image::{imageops, DynamicImage};
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
use std::{
    cell::Cell,
    env, mem,
    path::Path,
    rc::Rc,
    sync::{mpsc, Mutex, Once},
    thread,
};

static START: Once = Once::new();
static mut SENDER: Mutex<Option<mpsc::Sender<DynamicImage>>> = Mutex::new(None);

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
    Closed,
}

pub struct Surface {
    surface: wl_surface::WlSurface,
    layer_surface: Main<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1>,
    pool: AutoMemPool,
    dimensions: (u32, u32),
    image: Option<DynamicImage>,
}

impl Surface {
    fn new(
        output: &wl_output::WlOutput,
        surface: wl_surface::WlSurface,
        layer_shell: &Attached<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
        pool: AutoMemPool,
        image: Option<DynamicImage>,
    ) -> Self {
        let (width, height) = with_output_info(output, |info| {
            let dimensions = info.modes[0].dimensions;
            (dimensions.0 as u32, dimensions.1 as u32)
        })
        .unwrap();
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
                    next_render_event_handle.set(Some(RenderEvent::Closed));
                }
                (
                    zwlr_layer_surface_v1::Event::Configure {
                        serial,
                        width: _,
                        height: _,
                    },
                    next,
                ) if next != Some(RenderEvent::Closed) => {
                    layer_surface.ack_configure(serial);
                    next_render_event_handle.set(Some(RenderEvent::Redraw));
                }
                (_, _) => {}
            }
        });

        surface.commit();

        Self {
            surface,
            layer_surface,
            pool,
            dimensions: (width, height),
            image,
        }
    }

    fn draw(&mut self) {
        if let Some(image) = &self.image {
            let stride = 4 * self.dimensions.0 as i32;
            let width = self.dimensions.0 as i32;
            let height = self.dimensions.1 as i32;

            let (canvas, buffer) = self
                .pool
                .buffer(width, height, stride, wl_shm::Format::Argb8888)
                .unwrap();
            let image: Vec<u8> = image
                .resize_to_fill(width as u32, height as u32, imageops::FilterType::Lanczos3)
                .to_rgba8()
                .to_vec()
                .chunks_exact_mut(4)
                .flat_map(|pixel| {
                    pixel.swap(0, 2);
                    pixel.to_vec()
                })
                .collect();

            canvas.copy_from_slice(&image);

            self.image = None;
            mem::drop(image);

            self.surface.attach(Some(&buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, width as i32, height as i32);
            self.surface.commit();
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        self.layer_surface.destroy();
        self.surface.destroy();
    }
}

pub fn set_from_path<T>(path: T)
where
    T: AsRef<Path>,
{
    let image = image::open(path).expect("Failed to open image");
    START.call_once(|| {
        let (tx, rx) = mpsc::channel();
        unsafe {
            let mut sender = SENDER.lock().unwrap();
            *sender = Some(tx);
        }
        thread::spawn(
            || match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
                "wayland" => {
                    wayland(rx);
                }
                _ => panic!("Currently only wayland is supported"),
            },
        );
    });

    unsafe {
        let sender = SENDER.lock().unwrap();
        sender.as_ref().unwrap().send(image.into()).unwrap();
    }
}

pub fn set_from_memory<T>(image: T)
where
    T: Into<DynamicImage> + Send + 'static,
{
    START.call_once(|| {
        let (tx, rx) = mpsc::channel();
        unsafe {
            let mut sender = SENDER.lock().unwrap();
            *sender = Some(tx);
        }
        thread::spawn(
            || match env::var("XDG_SESSION_TYPE").unwrap_or_default().as_str() {
                "wayland" => {
                    wayland(rx);
                }
                _ => panic!("Currently only wayland is supported"),
            },
        );
    });

    unsafe {
        let sender = SENDER.lock().unwrap();
        sender.as_ref().unwrap().send(image.into()).unwrap();
    }
}

fn wayland(rx: mpsc::Receiver<DynamicImage>) {
    let (env, display, queue) =
        new_default_environment!(Env, fields = [layer_shell: SimpleGlobal::new(),])
            .expect("Initial roundtrip failed!");

    let surface = env.create_surface().detach();
    let pool = env
        .create_auto_pool()
        .expect("Failed to create a memory pool!");

    let layer_shell = env.require_global::<zwlr_layer_shell_v1::ZwlrLayerShellV1>();
    let output = env.get_all_outputs().first().unwrap().to_owned();
    let mut surface_wrapper = Surface::new(&output, surface, &layer_shell.clone(), pool, None);
    let mut event_loop = calloop::EventLoop::<()>::try_new().unwrap();
    WaylandSource::new(queue)
        .quick_insert(event_loop.handle())
        .unwrap();

    loop {
        display.flush().unwrap();
        event_loop.dispatch(None, &mut ()).unwrap();
        if let Ok(img) = rx.recv() {
            surface_wrapper.image = Some(img);
            surface_wrapper.draw();
        }
    }
}
