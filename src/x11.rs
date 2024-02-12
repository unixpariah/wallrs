use crate::helpers::resize_image;
use image::RgbImage;
use std::{error::Error, sync::mpsc};
use xcb::x;

pub fn x11(rx: mpsc::Receiver<RgbImage>) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let (width, height) = (screen.width_in_pixels(), screen.height_in_pixels());

    let window = screen.root();

    let pixmap: x::Pixmap = conn.generate_id();
    conn.check_request(conn.send_request_checked(&x::CreatePixmap {
        depth: screen.root_depth(),
        pid: pixmap,
        drawable: x::Drawable::Window(window),
        width,
        height,
    }))?;

    let gc = conn.generate_id();
    conn.check_request(conn.send_request_checked(&x::CreateGc {
        cid: gc,
        drawable: x::Drawable::Pixmap(pixmap),
        value_list: &[],
    }))?;

    loop {
        let image = rx.recv()?;
        let image = resize_image(image, width as u32, height as u32)?;
        conn.check_request(conn.send_request_checked(&x::PutImage {
            format: x::ImageFormat::ZPixmap,
            drawable: x::Drawable::Pixmap(pixmap),
            gc,
            width,
            height,
            dst_x: 0,
            dst_y: 0,
            left_pad: 0,
            depth: screen.root_depth(),
            data: &image,
        }))?;

        conn.check_request(conn.send_request_checked(&x::CopyArea {
            src_drawable: x::Drawable::Pixmap(pixmap),
            dst_drawable: x::Drawable::Window(window),
            gc,
            src_x: 0,
            src_y: 0,
            dst_x: 0,
            dst_y: 0,
            width,
            height,
        }))?;
    }
}
