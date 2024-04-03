use std::path::PathBuf;
use wlrs::{set_from_memory, set_from_path};

const TEST_IMG_DIR: &str = "tests/test_images";
const TEST_IMGS: [&str; 3] = [
    "tests/test_images/test1.jpg",
    "tests/test_images/test2.png",
    "tests/test_images/test3.bmp",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_test_imgs_test() {
        let p = PathBuf::from(TEST_IMG_DIR);
        if !p.is_dir() {
            std::fs::create_dir(p)
                .expect("Failed to create directory to put the images used for testing: ");
        }

        TEST_IMGS.iter().enumerate().for_each(|(i, test_img)| {
            let p = PathBuf::from(test_img);
            if !p.is_file() {
                let mut imgbuf =
                    image::ImageBuffer::new(400 * (i as u32 + 1), 400 * (i as u32 + 1));

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

    #[test]
    fn set_empty_image() {
        let img = image::RgbImage::new(0, 0);
        assert!(set_from_memory(img, vec![]).is_err());
    }

    #[test]
    fn set_images() {
        TEST_IMGS.iter().for_each(|test_img| {
            let img = image::open(test_img).unwrap();
            assert!(set_from_memory(img, vec![]).is_ok());
            assert!(set_from_path(test_img, vec![]).is_ok());
        });
    }

    #[test]
    fn set_image_that_does_not_exist() {
        assert!(set_from_path("", vec![]).is_err());
    }
}
