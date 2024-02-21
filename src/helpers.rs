use fast_image_resize::{FilterType, PixelType, Resizer};
use image::RgbImage;
use std::{error::Error, num::NonZeroU32};

/// Made it public just to be able to benchmark it but go ahead
/// Resize an image to the given width and height
///
/// Example:
/// ```rust
/// use image::RgbImage;
/// use wlrs::helpers::resize_image;
///
/// let image = RgbImage::new(1920, 1080);
/// let resized = resize_image(image, 1000, 1000).unwrap();
/// ```
/// Note, this function is totally ripped off from swww
pub fn resize_image(image: RgbImage, width: u32, height: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let (img_w, img_h) = image.dimensions();
    let image = image.into_vec();

    if img_w == width && img_h == height {
        return Ok(pad(
            image::RgbImage::from_raw(width, height, image).unwrap(),
            width,
            height,
        )?);
    }

    let ratio = width as f32 / height as f32;
    let img_r = img_w as f32 / img_h as f32;

    let (trg_w, trg_h) = if ratio > img_r {
        let scale = height as f32 / img_h as f32;
        ((img_w as f32 * scale) as u32, height as u32)
    } else {
        let scale = width as f32 / img_w as f32;
        (width as u32, (img_h as f32 * scale) as u32)
    };

    let trg_w = trg_w.min(width as u32);
    let trg_h = trg_h.min(height as u32);

    // If img_w, img_h, trg_w or trg_h is 0 you have bigger problems than unsafety
    let src = fast_image_resize::Image::from_vec_u8(
        unsafe { NonZeroU32::new_unchecked(img_w) },
        unsafe { NonZeroU32::new_unchecked(img_h) },
        image,
        PixelType::U8x3,
    )?;

    let new_w = unsafe { NonZeroU32::new_unchecked(trg_w) };
    let new_h = unsafe { NonZeroU32::new_unchecked(trg_h) };

    let mut dst = fast_image_resize::Image::new(new_w, new_h, PixelType::U8x3);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));

    resizer.resize(&src.view(), &mut dst_view)?;

    let dst = dst.into_vec();

    Ok(pad(
        image::RgbImage::from_raw(trg_w, trg_h, dst).unwrap(),
        width,
        height,
    )?)
}

pub fn pad(mut img: RgbImage, trg_w: u32, trg_h: u32) -> Result<Vec<u8>, String> {
    let color = [0, 0, 0];

    if img.dimensions() == (trg_w, trg_h) {
        return Ok(add_channel(img.into_vec()));
    }

    let (trg_w, trg_h) = (trg_w as usize, trg_h as usize);
    let mut padded = Vec::with_capacity(trg_w * trg_h * 3);

    let img = image::imageops::crop(&mut img, 0, 0, trg_w as u32, trg_h as u32).to_image();
    let (img_w, img_h) = img.dimensions();
    let (img_w, img_h) = (img_w as usize, img_h as usize);
    let raw_img = img.into_vec();

    (0..(((trg_h - img_h) / 2) * trg_w)).for_each(|_| {
        padded.push(color[2]);
        padded.push(color[1]);
        padded.push(color[0]);
    });

    let left_border_w = (trg_w - img_w) / 2;
    let right_border_w = left_border_w + (img_w % 2);

    (0..img_h).for_each(|row| {
        (0..left_border_w).for_each(|_| {
            padded.push(color[2]);
            padded.push(color[1]);
            padded.push(color[0]);
        });

        raw_img[(row * img_w * 3)..((row + 1) * img_w * 3)]
            .chunks_exact(3)
            .for_each(|pixel| {
                padded.push(pixel[2]);
                padded.push(pixel[1]);
                padded.push(pixel[0]);
            });

        (0..right_border_w).for_each(|_| {
            padded.push(color[2]);
            padded.push(color[1]);
            padded.push(color[0]);
        });
    });

    while padded.len() < (trg_w * trg_h * 3) {
        padded.push(color[2]);
        padded.push(color[1]);
        padded.push(color[0]);
    }

    Ok(add_channel(padded))
}

fn add_channel(dst: Vec<u8>) -> Vec<u8> {
    let mut rgba_dst = Vec::with_capacity(dst.len() / 3 * 4);
    dst.chunks(3).for_each(|rgb_pixels| {
        let pixel = [rgb_pixels[2], rgb_pixels[1], rgb_pixels[0], 255];
        rgba_dst.extend_from_slice(&pixel);
    });

    rgba_dst
}
