use crate::helpers::resize_image;
use image::RgbaImage;
use std::sync::mpsc;
use xcb::x;

pub fn xorg(rx: mpsc::Receiver<RgbaImage>) -> xcb::Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let (width, height) = (screen.width_in_pixels(), screen.height_in_pixels());

    let window = screen.root();

    conn.send_request(&x::ChangeWindowAttributes {
        window,
        value_list: &[x::Cw::EventMask(x::EventMask::NO_EVENT)],
    });

    let pixmap: x::Pixmap = conn.generate_id();
    conn.send_request(&x::CreatePixmap {
        depth: screen.root_depth(),
        pid: pixmap,
        drawable: x::Drawable::Pixmap(pixmap),
        width,
        height,
    });

    let gc: x::Gcontext = conn.generate_id();
    conn.send_request(&x::CreateGc {
        cid: gc,
        drawable: x::Drawable::Pixmap(pixmap),
        value_list: &[],
    });

    conn.send_request(&x::MapWindow { window });

    conn.flush()?;

    loop {
        let image = rx.recv().unwrap();
        let image = resize_image(image, width as u32, height as u32).unwrap();
        conn.send_request(&x::PutImage {
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
        });

        conn.flush()?;

        conn.send_request(&x::CopyArea {
            src_drawable: x::Drawable::Pixmap(pixmap),
            dst_drawable: x::Drawable::Window(window),
            gc,
            src_x: 0,
            src_y: 0,
            dst_x: 0,
            dst_y: 0,
            width,
            height,
        });

        conn.flush()?;
    }
}
