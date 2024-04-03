use crate::{helpers::resize_image, WallpaperData};
use std::{error::Error, sync::mpsc};
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
    width: u16,
    height: u16,
    x: i16,
    y: i16,
}

pub fn x11(
    rx: mpsc::Receiver<WallpaperData>,
    tx: mpsc::Sender<bool>,
) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = match connect(None) {
        Ok((conn, screen_num)) => (conn, screen_num),
        Err(_) => {
            _ = tx.send(false);
            return Err("Failed to connect to x11 server".into());
        }
    };
    let screen = conn.setup().roots[screen_num].to_owned();
    let res = conn
        .randr_get_screen_resources_current(screen.root)?
        .reply()?;

    let screens: Vec<Screen> = res
        .outputs
        .iter()
        .filter_map(|output| {
            let crtc = conn
                .randr_get_output_info(*output, res.config_timestamp)
                .ok()?
                .reply()
                .ok()?
                .crtc;

            if crtc == 0 {
                return None;
            }

            let crtc_info = conn
                .randr_get_crtc_info(crtc, res.config_timestamp)
                .ok()?
                .reply()
                .ok()?;

            let screen = Screen {
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
        screens.iter().enumerate().try_for_each(|(index, scr)| {
            if wallpaper_data.output_num.contains(&(index as u8))
                || wallpaper_data.output_num.is_empty()
            {
                let image =
                    resize_image(&wallpaper_data.image, scr.width as u32, scr.height as u32)
                        .map_err(|e| e.to_string())?;
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

            Ok::<(), Box<dyn Error>>(())
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
    }
}
