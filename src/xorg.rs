use crate::helpers::resize_image;
use image::RgbaImage;
use std::sync::mpsc;
use xcb::x;

pub fn xorg(rx: mpsc::Receiver<RgbaImage>) -> xcb::Result<()> {
    let rectangles: &[x::Rectangle] = &[x::Rectangle {
        x: 200,
        y: 200,
        width: 400,
        height: 400,
    }];
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    let (width, height) = (screen.width_in_pixels(), screen.height_in_pixels());

    let window = screen.root();

    conn.send_request(&x::ChangeWindowAttributes {
        window,
        value_list: &[
            x::Cw::BackPixel(screen.white_pixel()),
            x::Cw::EventMask(x::EventMask::EXPOSURE),
        ],
    });

    let gc: x::Gcontext = conn.generate_id();
    conn.send_request(&x::CreateGc {
        cid: gc,
        drawable: x::Drawable::Window(window),
        value_list: &[
            x::Gc::Foreground(screen.black_pixel()),
            x::Gc::Background(screen.white_pixel()),
            x::Gc::LineWidth(1),
            x::Gc::LineStyle(x::LineStyle::OnOffDash),
            x::Gc::SubwindowMode(x::SubwindowMode::IncludeInferiors),
        ],
    });

    conn.send_request(&x::MapWindow { window });

    conn.flush()?;
    loop {
        let image = rx.recv().unwrap();
        let _image = resize_image(image, width as u32, height as u32);
        let event = match conn.wait_for_event() {
            Err(xcb::Error::Connection(xcb::ConnError::Connection)) => {
                break Ok(());
            }
            Err(err) => {
                panic!("unexpected error: {:#?}", err);
            }
            Ok(event) => event,
        };
        match event {
            xcb::Event::X(x::Event::Expose(_ev)) => {
                let drawable = x::Drawable::Window(window);

                conn.send_request(&x::PolyRectangle {
                    drawable,
                    gc,
                    rectangles,
                });

                conn.flush()?;
            }
            _ => {}
        }
    }
}
