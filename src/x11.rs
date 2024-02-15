use crate::helpers::resize_image;
use image::*;
use std::{error::Error, sync::mpsc};
use xcb_util as xcbu;

const ATOMS: &'static [&'static str] = &["_XROOTPMAP_ID", "_XSETROOT_ID", "ESETROOT_PMAP_ID"];

pub fn x11(rx: mpsc::Receiver<RgbImage>) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = xcb::Connection::connect(None).expect("Failed to connect to X server");

    let setup = &conn.get_setup();
    let mut screen_iter = setup.roots();
    let screen = screen_iter
        .nth(screen_num as usize)
        .expect("Failed to get screen info");

    let w = screen.width_in_pixels();
    let h = screen.height_in_pixels();

    let mut shm =
        xcbu::image::shm::create(&conn, screen.root_depth(), w, h).expect("Failed to create SHM");

    let pixmap_id = conn.generate_id();
    xcb::create_pixmap(&conn, screen.root_depth(), pixmap_id, screen.root(), w, h);

    let context = conn.generate_id();
    xcb::create_gc(&conn, context, pixmap_id, &[]);

    loop {
        let image = rx.recv()?;
        let image = resize_image(image, w as u32, h as u32)?;
        let mut x = 0;
        let mut y = 0;

        image.chunks_exact(4).for_each(|pixel| {
            let r = pixel[2];
            let g = pixel[1];
            let b = pixel[0];
            shm.put(x, y, (r as u32) << 16 | (g as u32) << 8 | (b as u32) << 0);

            x += 1;
            if x == w as u32 {
                x = 0;
                y += 1;
            }
        });

        xcbu::image::shm::put(
            &conn, pixmap_id, context, &shm, 0, 0, 0, 0, w as u16, h as u16, false,
        )
        .expect("Failed to draw to pixmap");

        xcb::kill_client(&conn, xcb::KILL_ALL_TEMPORARY);
        xcb::set_close_down_mode(&conn, xcb::CLOSE_DOWN_RETAIN_TEMPORARY as u8);
        for atom in ATOMS {
            xcb::change_property(
                &conn,
                xcb::PROP_MODE_REPLACE as u8,
                screen.root(),
                xcb::intern_atom(&conn, false, atom)
                    .get_reply()
                    .expect("failed to intern atom")
                    .atom(),
                xcb::ATOM_PIXMAP,
                32,
                &[pixmap_id],
            );
        }

        xcb::kill_client(&conn, xcb::KILL_ALL_TEMPORARY);
        xcb::set_close_down_mode(&conn, xcb::CLOSE_DOWN_RETAIN_TEMPORARY as u8);
        xcb::change_window_attributes(&conn, screen.root(), &[(xcb::CW_BACK_PIXMAP, pixmap_id)]);
        xcb::clear_area(&conn, false, screen.root(), 0, 0, w, h);
        let _ = &conn.flush();
    }
}
