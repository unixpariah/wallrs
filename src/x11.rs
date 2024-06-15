use crate::{error::WlrsError, helpers::resize, WallpaperData};
use rayon::prelude::*;
use std::sync::mpsc;
use x11rb::{
    connect,
    connection::Connection,
    protocol::{
        randr::ConnectionExt as _,
        xproto::{
            AtomEnum, ChangeWindowAttributesAux, CloseDown, ConnectionExt as _, CreateGCAux,
            ImageFormat, Kill, PropMode,
        },
    },
    wrapper::ConnectionExt as _,
};

const ATOMS: &[&str] = &["_XROOTPMAP_ID", "_XSETROOT_ID", "ESETROOT_PMAP_ID"];

struct Screen {
    name: String,
    width: u16,
    height: u16,
    x: i16,
    y: i16,
}

pub fn x11(
    rx: mpsc::Receiver<WallpaperData>,
    tx: mpsc::Sender<Result<(), WlrsError>>,
) -> Result<(), WlrsError> {
    let (conn, screen_num) = connect(None)?;
    let screen = conn.setup().roots[screen_num].to_owned();
    let res = conn
        .randr_get_screen_resources_current(screen.root)?
        .reply()?;

    let screens: Vec<Screen> = res
        .outputs
        .iter()
        .filter_map(|output| {
            let output_info = conn
                .randr_get_output_info(*output, res.config_timestamp)
                .ok()?
                .reply()
                .ok()?;

            let crtc = output_info.crtc;

            if crtc == 0 {
                return None;
            }

            let crtc_info = conn
                .randr_get_crtc_info(crtc, res.config_timestamp)
                .ok()?
                .reply()
                .ok()?;

            let name = String::from_utf8(output_info.name.to_vec()).ok()?;

            let screen = Screen {
                name,
                width: crtc_info.width,
                height: crtc_info.height,
                x: crtc_info.x,
                y: crtc_info.y,
            };

            Some(screen)
        })
        .collect();

    let width = screen.width_in_pixels;
    let height = screen.height_in_pixels;

    let pixmap = conn.generate_id()?;
    conn.create_pixmap(screen.root_depth, pixmap, screen.root, width, height)?;

    let context = conn.generate_id()?;
    conn.create_gc(context, pixmap, &CreateGCAux::new())?;
    conn.flush()?;

    loop {
        let wallpaper_data = rx.recv()?;
        screens.par_iter().try_for_each(|scr| {
            if wallpaper_data.outputs.contains(&scr.name) || wallpaper_data.outputs.is_empty() {
                let image = resize(&wallpaper_data, [width as i32, height as i32]).unwrap();
                let _ = conn.put_image(
                    ImageFormat::Z_PIXMAP,
                    pixmap,
                    context,
                    scr.width,
                    scr.height,
                    scr.x,
                    scr.y,
                    0,
                    screen.root_depth,
                    &image,
                );
            }

            Ok::<(), WlrsError>(())
        })?;
        conn.kill_client(Kill::ALL_TEMPORARY)?;
        conn.set_close_down_mode(CloseDown::RETAIN_TEMPORARY)?;
        ATOMS.iter().for_each(|atom| {
            if let Ok(intern_atom_cookie) = conn.intern_atom(false, atom.as_bytes()) {
                if let Ok(intern_atom_reply) = intern_atom_cookie.reply() {
                    let atom = intern_atom_reply.atom;
                    let _ = conn.change_property32(
                        PropMode::REPLACE,
                        screen.root,
                        atom,
                        AtomEnum::PIXMAP,
                        &[pixmap],
                    );
                };
            }
        });

        conn.kill_client(Kill::ALL_TEMPORARY)?;
        conn.set_close_down_mode(CloseDown::RETAIN_TEMPORARY)?;
        conn.change_window_attributes(
            screen.root,
            &ChangeWindowAttributesAux::new().background_pixmap(pixmap),
        )?;
        conn.clear_area(false, screen.root, 0, 0, width, height)?;
        _ = tx.send(Ok(()));
    }
}
