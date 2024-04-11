use std::path::PathBuf;
use wlrs::{set_from_memory, set_from_path, CropMode};

const TEST_IMG_DIR: &str = "tests/test_images";
const TEST_IMGS: [&str; 3] = [
    "tests/test_images/test1.jpg",
    "tests/test_images/test2.png",
    "tests/test_images/test3.bmp",
];

fn make_test_imgs() {
    let images_dir = PathBuf::from(TEST_IMG_DIR);
    if !images_dir.exists() {
        std::fs::create_dir(images_dir)
            .expect("Failed to create directory to put the images used for testing: ");
    }

    TEST_IMGS.iter().enumerate().for_each(|(i, test_img)| {
        let img_path = PathBuf::from(test_img);
        if !img_path.exists() {
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

fn set_empty_image() {
    let img = image::RgbImage::new(0, 0);
    assert!(set_from_memory(img, vec![], CropMode::Fit(None)).is_err());
}

fn set_images_from_path() {
    TEST_IMGS.iter().for_each(|test_img| {
        assert!(set_from_path(test_img, vec![0], CropMode::Fit(Some([255, 0, 0]))).is_ok());
        assert!(set_from_path(test_img, vec![1], CropMode::No(Some([255, 0, 0]))).is_ok());
        assert!(set_from_path(test_img, vec![], CropMode::Crop).is_ok());
    });
}

fn set_images_from_memory() {
    TEST_IMGS.iter().for_each(|test_img| {
        let img = image::open(test_img).unwrap();
        assert!(set_from_memory(img.clone(), vec![0], CropMode::Fit(Some([255, 0, 0]))).is_ok());
        assert!(set_from_memory(img.clone(), vec![1], CropMode::No(Some([255, 0, 0]))).is_ok());
        assert!(set_from_memory(img, vec![], CropMode::Crop).is_ok());
    });
}

fn set_image_that_does_not_exist() {
    assert!(set_from_path("", vec![], CropMode::Fit(None)).is_err());
}

fn set_image_after_error() {
    let img = image::RgbImage::new(0, 0);
    assert!(set_from_memory(img, vec![], CropMode::Fit(None)).is_err());
    let img = image::open(TEST_IMGS[0]).unwrap();
    assert!(set_from_memory(img, vec![], CropMode::Fit(None)).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration() {
        make_test_imgs();

        set_empty_image();
        set_images_from_path();
        set_images_from_memory();
        set_image_that_does_not_exist();
        set_image_after_error();
    }
}
