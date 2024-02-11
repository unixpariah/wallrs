use std::path::PathBuf;
use wlrs::{set_from_memory, set_from_path};

const TEST_IMG_DIR: &str = "tests/test_images";
const TEST_IMGS: [&str; 3] = [
    "tests/test_images/test1.jpg",
    "tests/test_images/test2.png",
    "tests/test_images/test2.bmp",
];

fn make_img_dir() {
    let p = PathBuf::from(TEST_IMG_DIR);
    if !p.is_dir() {
        std::fs::create_dir(p)
            .expect("Failed to create directory to put the images used for testing: ");
    }
}

fn make_test_imgs() {
    make_img_dir();
    TEST_IMGS.iter().enumerate().for_each(|(i, test_img)| {
        let p = PathBuf::from(test_img);
        if !p.is_file() {
            let mut imgbuf = image::ImageBuffer::new(400 * (i as u32 + 1), 400 * (i as u32 + 1));

            for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
                let r = (0.3 * x as f32) as u8;
                let b = (0.3 * y as f32) as u8;
                *pixel = image::Rgb([r, 0, b]);
            }

            imgbuf
                .save(test_img)
                .expect("Failed to create image for testing: ");
        }
    })
}

fn set_images() {
    TEST_IMGS.iter().for_each(|test_img| {
        let img = image::open(test_img).unwrap();
        assert!(set_from_memory(img).is_ok());
        assert!(set_from_path(test_img).is_ok());
    });
}

fn set_image_that_does_not_exist() {
    assert!(set_from_path("").is_err());
}

#[test]
fn integration_tests() {
    make_test_imgs();

    set_images();
    set_image_that_does_not_exist();
}
