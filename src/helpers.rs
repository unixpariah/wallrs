use fast_image_resize::{FilterType, PixelType, Resizer};
use image::RgbImage;
use std::{error::Error, num::NonZeroU32};

pub(crate) fn resize_image(
    image: &RgbImage,
    width: u32,
    height: u32,
    color: [u8; 3],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let (img_w, img_h) = image.dimensions();
    let image = image.as_raw().to_vec();

    if img_w == width && img_h == height {
        return pad(
            &mut image::RgbImage::from_raw(width, height, image).ok_or("")?,
            width,
            height,
            color,
        );
    }

    let ratio = width as f32 / height as f32;
    let img_r = img_w as f32 / img_h as f32;

    let (trg_w, trg_h) = if ratio > img_r {
        let scale = height as f32 / img_h as f32;
        ((img_w as f32 * scale) as u32, height)
    } else {
        let scale = width as f32 / img_w as f32;
        (width, (img_h as f32 * scale) as u32)
    };

    let trg_w = trg_w.min(width);
    let trg_h = trg_h.min(height);

    let src = fast_image_resize::Image::from_vec_u8(
        NonZeroU32::new(img_w).ok_or("")?,
        NonZeroU32::new(img_h).ok_or("")?,
        image,
        PixelType::U8x3,
    )?;

    let new_w = NonZeroU32::new(trg_w).ok_or("")?;
    let new_h = NonZeroU32::new(trg_h).ok_or("")?;

    let mut dst = fast_image_resize::Image::new(new_w, new_h, PixelType::U8x3);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));

    resizer.resize(&src.view(), &mut dst_view)?;

    let dst = dst.into_vec();

    pad(
        &mut image::RgbImage::from_raw(trg_w, trg_h, dst).ok_or("")?,
        width,
        height,
        color,
    )
}

pub(crate) fn pad(
    img: &mut RgbImage,
    trg_w: u32,
    trg_h: u32,
    color: [u8; 3],
) -> Result<Vec<u8>, Box<dyn Error>> {
    if img.dimensions() == (trg_w, trg_h) {
        return Ok(img.to_vec());
    }

    let (trg_w, trg_h) = (trg_w as usize, trg_h as usize);
    let mut padded = Vec::with_capacity(trg_w * trg_h * 3);

    let img = image::imageops::crop(img, 0, 0, trg_w as u32, trg_h as u32).to_image();
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

    Ok(padded)
}
