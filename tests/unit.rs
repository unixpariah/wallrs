use image::RgbImage;
use wlrs::wayland;

const TEST_IMGS: [&str; 3] = [
    "tests/test_images/test1.jpg",
    "tests/test_images/test2.png",
    "tests/test_images/test2.bmp",
];

#[test]
fn shrink_images() {
    TEST_IMGS.iter().enumerate().for_each(|(i, image_path)| {
        let img = image::open(image_path).unwrap().to_rgb8();
        let (w, h) = img.dimensions();
        let (new_w, new_h) = (w * (i as u32 + 1), h * (i as u32 + 1));
        let resized_img = wayland::resize_image(img, new_w, new_h).unwrap();
        let img = RgbImage::from_vec(new_w, new_h, resized_img).unwrap();
        assert_eq!(img.dimensions(), (new_w, new_h));
    });
}

#[test]
fn enlarge_images() {
    TEST_IMGS.iter().enumerate().for_each(|(i, image_path)| {
        let img = image::open(image_path).unwrap().to_rgb8();
        let (w, h) = img.dimensions();
        let (new_w, new_h) = (w * (i as u32 + 1), h * (i as u32 + 1));
        let resized_img = wayland::resize_image(img, new_w, new_h).unwrap();
        let img = RgbImage::from_vec(new_w, new_h, resized_img).unwrap();
        assert_eq!(img.dimensions(), (new_w, new_h));
    });
}
