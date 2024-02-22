use crate::helpers::resize_image;
use image::RgbImage;
use std::{error::Error, sync::mpsc};
use x11rb::{
    connect,
    connection::Connection,
    protocol::xproto::{
        AtomEnum, ChangeWindowAttributesAux, CloseDown, ConnectionExt, CreateGCAux, ImageFormat,
        Kill, PropMode,
    },
    wrapper::ConnectionExt as _,
};

const ATOMS: &[&str] = &["_XROOTPMAP_ID", "_XSETROOT_ID", "ESETROOT_PMAP_ID"];

pub fn x11(rx: mpsc::Receiver<RgbImage>) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = connect(None).expect("Failed to connect to X server");
    let screen = conn.setup().roots[screen_num].to_owned();

    let width = screen.width_in_pixels;
    let height = screen.height_in_pixels;

    let pixmap = conn.generate_id()?;
    conn.create_pixmap(screen.root_depth, pixmap, screen.root, width, height)?;

    let context = conn.generate_id()?;
    conn.create_gc(context, pixmap, &CreateGCAux::new())?;
    conn.flush()?;

    loop {
        let image = rx.recv()?;
        let image = resize_image(&image, width as u32, height as u32)?;

        conn.put_image(
            ImageFormat::Z_PIXMAP,
            pixmap,
            context,
            width,
            height,
            0,
            0,
            0,
            screen.root_depth,
            &image,
        )?;
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
