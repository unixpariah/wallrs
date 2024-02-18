use crate::helpers::resize_image;
use image::*;
use std::{error::Error, sync::mpsc};
use xcb::{shm, x};

const ATOMS: &'static [&'static str] = &["_XROOTPMAP_ID", "_XSETROOT_ID", "ESETROOT_PMAP_ID"];

pub fn x11(rx: mpsc::Receiver<RgbImage>) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = xcb::Connection::connect(None).expect("Failed to connect to X server");

    let setup = &conn.get_setup();
    let mut screen_iter = setup.roots();
    let screen = screen_iter
        .nth(screen_num as usize)
        .expect("Failed to get screen info");

    let width = screen.width_in_pixels();
    let height = screen.height_in_pixels();

    // Create shared memory

    let pixmap = conn.generate_id();
    conn.check_request(conn.send_request_checked(&x::CreatePixmap {
        depth: screen.root_depth(),
        pid: pixmap,
        drawable: x::Drawable::Window(screen.root()),
        width,
        height,
    }))?;

    let context = conn.generate_id();
    conn.check_request(conn.send_request_checked(&x::CreateGc {
        cid: context,
        drawable: x::Drawable::Pixmap(pixmap),
        value_list: &[],
    }))?;

    loop {
        let image = rx.recv()?;
        let image = resize_image(image, width as u32, height as u32)?;

        // Put the image onto the shared memory segment

        conn.check_request(conn.send_request_checked(&x::KillClient {
            resource: x::Kill::AllTemporary as u32,
        }))?;

        conn.check_request(conn.send_request_checked(&x::SetCloseDownMode {
            mode: x::CloseDown::RetainTemporary,
        }))?;

        ATOMS.iter().for_each(|atom| {
            let atom = conn
                .wait_for_reply(conn.send_request(&x::InternAtom {
                    only_if_exists: false,
                    name: atom.as_bytes(),
                }))
                .unwrap()
                .atom();

            conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window: screen.root(),
                property: atom,
                r#type: x::ATOM_PIXMAP,
                data: &[screen.root()],
            });
        });

        conn.check_request(conn.send_request_checked(&x::KillClient {
            resource: x::Kill::AllTemporary as u32,
        }))?;
        conn.check_request(conn.send_request_checked(&x::SetCloseDownMode {
            mode: x::CloseDown::RetainTemporary,
        }))?;
        conn.check_request(conn.send_request_checked(&x::ChangeWindowAttributes {
            window: screen.root(),
            value_list: &[x::Cw::BackPixmap(pixmap)],
        }))?;
        conn.check_request(conn.send_request_checked(&x::ClearArea {
            exposures: false,
            window: screen.root(),
            x: 0,
            y: 0,
            width,
            height,
        }))?;
        conn.flush()?;
    }
}
