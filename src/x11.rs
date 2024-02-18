use crate::helpers::resize_image;
use image::RgbImage;
use std::{
    error::Error,
    fs::{remove_file, OpenOptions},
    io::Write,
    sync::mpsc,
};
use x11rb::{
    connect,
    connection::Connection,
    protocol::{
        shm,
        xproto::{
            AtomEnum, ChangeWindowAttributesAux, CloseDown, ConnectionExt, CreateGCAux, Kill,
            PropMode,
        },
    },
};

const ATOMS: &'static [&'static str] = &["_XROOTPMAP_ID", "_XSETROOT_ID", "ESETROOT_PMAP_ID"];
const TEMP_FILE_CONTENT: [u8; 8] = [0x00, 0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc];

fn make_file() -> std::io::Result<std::fs::File> {
    let file_name = "shared_memory.bin";
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open(file_name)?;
    file.write_all(&TEMP_FILE_CONTENT)?;
    remove_file(file_name)?;
    Ok(file)
}

pub fn x11(rx: mpsc::Receiver<RgbImage>) -> Result<(), Box<dyn Error>> {
    let file = make_file()?;
    let (conn, screen_num) = connect(None).expect("Failed to connect to X server");
    let screen = conn.setup().roots[screen_num].clone();

    let width = screen.width_in_pixels;
    let height = screen.height_in_pixels;

    let shmseg = conn.generate_id()?;
    shm::create_segment(&conn, shmseg, width as u32 * height as u32 * 4, false)?;
    shm::attach_fd(&conn, shmseg, file, false)?;

    let pixmap = conn.generate_id()?;
    shm::create_pixmap(
        &conn,
        pixmap,
        screen.root,
        width,
        height,
        screen.root_depth,
        shmseg,
        0,
    )?;

    let context = conn.generate_id()?;
    conn.create_gc(context, pixmap, &CreateGCAux::new())?;
    conn.flush()?;

    loop {
        let image = rx.recv()?;
        let _image = resize_image(image, width as u32, height as u32)?;

        // Put image into shared memory

        conn.kill_client(Kill::ALL_TEMPORARY)?;
        conn.set_close_down_mode(CloseDown::RETAIN_TEMPORARY)?;

        ATOMS.iter().for_each(|atom| {
            let atom = conn
                .intern_atom(false, atom.as_bytes())
                .unwrap()
                .reply()
                .unwrap()
                .atom;

            conn.change_property(
                PropMode::REPLACE,
                screen.root,
                atom,
                AtomEnum::PIXMAP,
                32,
                1,
                &[screen.root as u8],
            )
            .unwrap();
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
