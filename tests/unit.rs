use image::RgbImage;
use wlrs::helpers::resize_image;

const TEST_IMGS: [&str; 3] = [
    "tests/test_images/test1.jpg",
    "tests/test_images/test2.png",
    "tests/test_images/test2.bmp",
];

#[test]
fn resize_images() {
    TEST_IMGS.iter().enumerate().for_each(|(i, image_path)| {
        let img = image::open(image_path).unwrap().to_rgb8();
        let (w, h) = img.dimensions();
        let (new_w, new_h) = (w * (i as u32 + 1), h * (i as u32 + 1));
        let resized_img = resize_image(img, new_w, new_h).unwrap();
        let img = RgbImage::from_vec(new_w, new_h, resized_img.clone()).unwrap();
        assert_eq!(img.dimensions(), (new_w, new_h));
        // Check if image has 4 channels per pixel
        assert_eq!(resized_img.len(), new_w as usize * new_h as usize * 4);
    });
}
