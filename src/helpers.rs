use fast_image_resize::{FilterType, PixelType, Resizer};
use image::RgbImage;
use std::{error::Error, num::NonZeroU32};

pub fn resize_image(image: RgbImage, width: u32, height: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let (img_w, img_h) = image.dimensions();

    if img_w == width && img_h == height {
        return Ok(add_channel(image.into_raw()));
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
        image.into_raw(),
        PixelType::U8x3,
    )?;

    let trg_w = unsafe { NonZeroU32::new_unchecked(trg_w) };
    let trg_h = unsafe { NonZeroU32::new_unchecked(trg_h) };

    let mut dst = fast_image_resize::Image::new(trg_w, trg_h, PixelType::U8x3);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));

    resizer.resize(&src.view(), &mut dst_view)?;

    let dst = dst.into_vec();

    Ok(add_channel(dst))
}

fn add_channel(dst: Vec<u8>) -> Vec<u8> {
    let mut rgba_dst = Vec::with_capacity(dst.len() / 3 * 4);
    dst.chunks(3).for_each(|rgb_pixels| {
        rgba_dst.extend_from_slice(rgb_pixels);
        rgba_dst.push(255);
    });

    rgba_dst
}
