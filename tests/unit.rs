use wlrs::helpers::resize_image;

#[test]
fn test_resize_image() {
    let img = image::RgbImage::new(100, 100);

    let resized_img = resize_image(img.clone(), 200, 200);
    assert!(resized_img.is_ok());
    assert_eq!(resized_img.unwrap().len(), 200 * 200 * 4);

    let resized_img = resize_image(img.clone(), 50, 50);
    assert!(resized_img.is_ok());
    assert_eq!(resized_img.unwrap().len(), 50 * 50 * 4);

    let resized_img = resize_image(img.clone(), 100, 100);
    assert!(resized_img.is_ok());
    assert_eq!(resized_img.unwrap().len(), 100 * 100 * 4);

    let resized_img = resize_image(img.clone(), 200, 150);
    assert!(resized_img.is_ok());
    assert_eq!(resized_img.unwrap().len(), 200 * 150 * 4);

    let resized_img = resize_image(img.clone(), 150, 200);
    assert!(resized_img.is_ok());
    assert_eq!(resized_img.unwrap().len(), 150 * 200 * 4);
}
