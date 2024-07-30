use std::{num::NonZeroU32, path::PathBuf};
use wlrs::{CropMode, Image, SetType, Wlrs};

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
                *pixel = image::Rgba([r, 0, b, 255]);
            }

            imgbuf
                .save(test_img)
                .expect("Failed to create image for testing: ");
        }
    })
}

fn set_images_from_path() {
    let wlrs = Wlrs::new();
    assert!(wlrs.is_ok());
    let wlrs = wlrs.unwrap();

    TEST_IMGS.iter().for_each(|test_img| {
        assert!(wlrs
            .set(
                SetType::Path(test_img),
                &["eDP-1".to_string()],
                CropMode::Fit(Some([255, 0, 0]))
            )
            .is_ok());
        assert!(wlrs
            .set(
                SetType::Path(test_img),
                &["HDMI-A-1".to_string()],
                CropMode::No(Some([255, 0, 0]))
            )
            .is_ok());
        assert!(wlrs
            .set(
                SetType::Path(test_img),
                &["eDP-1".to_string()],
                CropMode::Crop
            )
            .is_ok());
    });
}

fn set_images_from_memory() {
    let wlrs = Wlrs::new().unwrap();
    TEST_IMGS.iter().for_each(|test_img| {
        let img = image::open(test_img).unwrap();

        {
            let img = Image::new(
                &img.to_rgba8(),
                NonZeroU32::new(img.width()).unwrap(),
                NonZeroU32::new(img.height()).unwrap(),
            )
            .unwrap();
            assert!(wlrs
                .set(
                    SetType::Img(img),
                    &["eDP-1".to_string()],
                    CropMode::Fit(Some([255, 0, 0]))
                )
                .is_ok());
        }

        {
            let img = Image::new(
                &img.to_rgba8(),
                NonZeroU32::new(img.width()).unwrap(),
                NonZeroU32::new(img.height()).unwrap(),
            )
            .unwrap();
            assert!(wlrs
                .set(
                    SetType::Img(img),
                    &["HDMI-A-1".to_string()],
                    CropMode::No(Some([255, 0, 0]))
                )
                .is_ok());
        }

        {
            let img = Image::new(
                &img.to_rgba8(),
                NonZeroU32::new(img.width()).unwrap(),
                NonZeroU32::new(img.height()).unwrap(),
            )
            .unwrap();
            assert!(wlrs.set(SetType::Img(img), &[], CropMode::Crop).is_ok());
        }
    });
}

fn set_image_that_does_not_exist() {
    let wlrs = Wlrs::new().unwrap();
    assert!(wlrs
        .set(SetType::Path(""), &[], CropMode::Fit(None))
        .is_err());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration() {
        make_test_imgs();

        set_images_from_path();
        set_images_from_memory();
        set_image_that_does_not_exist();
    }
}
