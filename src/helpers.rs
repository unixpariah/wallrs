use fast_image_resize::{FilterType, PixelType, Resizer};
use image::RgbImage;
use std::num::NonZeroU32;

use crate::error::WlrsError;

pub(crate) fn resize_image(
    image: &RgbImage,
    width: u32,
    height: u32,
    color: [u8; 3],
) -> Result<Vec<u8>, WlrsError> {
    let (img_w, img_h) = image.dimensions();
    let image = image.as_raw().to_vec();

    if img_w == width && img_h == height {
        return pad(
            &image::RgbImage::from_raw(width, height, image).ok_or(WlrsError::CustomError(""))?,
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
        NonZeroU32::new(img_w)
            .ok_or(WlrsError::SizeError("Width of image must be bigger than 0"))?,
        NonZeroU32::new(img_h).ok_or(WlrsError::SizeError(
            "Height of image must be bigger than 0",
        ))?,
        image,
        PixelType::U8x3,
    )?;

    let new_w =
        NonZeroU32::new(trg_w).ok_or(WlrsError::SizeError("Your monitor has 0 width (???)"))?;
    let new_h =
        NonZeroU32::new(trg_h).ok_or(WlrsError::SizeError("Your monitor has 0 height (???)"))?;

    let mut dst = fast_image_resize::Image::new(new_w, new_h, PixelType::U8x3);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));

    resizer.resize(&src.view(), &mut dst_view)?;

    let dst = dst.into_vec();

    pad(
        &image::RgbImage::from_raw(trg_w, trg_h, dst).ok_or(WlrsError::CustomError(""))?,
        width,
        height,
        color,
    )
}

pub(crate) fn pad(
    img: &RgbImage,
    trg_w: u32,
    trg_h: u32,
    color: [u8; 3],
) -> Result<Vec<u8>, WlrsError> {
    if img.dimensions() == (trg_w, trg_h) {
        return Ok(img.to_vec());
    }

    let (trg_w, trg_h) = (trg_w as usize, trg_h as usize);
    let mut padded = Vec::with_capacity(trg_w * trg_h * 3);

    let img = image::imageops::crop_imm(img, 0, 0, trg_w as u32, trg_h as u32).to_image();
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

pub fn crop_image(img: &RgbImage, width: u32, height: u32) -> Result<Vec<u8>, WlrsError> {
    if (img.width(), img.height()) == (width, height) {
        return Ok(img.to_vec());
    }

    let pixel_type = PixelType::U8x3;
    let src = fast_image_resize::Image::from_vec_u8(
        NonZeroU32::new(img.width()).ok_or(WlrsError::SizeError("Empty image"))?,
        NonZeroU32::new(img.height()).ok_or(WlrsError::SizeError("Empty image"))?,
        img.to_vec(),
        pixel_type,
    )?;

    // We unwrap below because we know the outputs's dimensions should never be 0
    let new_w = NonZeroU32::new(width).ok_or(WlrsError::SizeError("Empty image"))?;
    let new_h = NonZeroU32::new(height).ok_or(WlrsError::SizeError("Empty image"))?;
    let mut src_view = src.view();
    src_view.set_crop_box_to_fit_dst_size(new_w, new_h, Some((0.5, 0.5)));

    let mut dst = fast_image_resize::Image::new(new_w, new_h, pixel_type);
    let mut dst_view = dst.view_mut();

    let mut resizer = Resizer::new(fast_image_resize::ResizeAlg::Convolution(
        FilterType::Lanczos3,
    ));
    resizer.resize(&src_view, &mut dst_view)?;

    Ok(dst.into_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_image() {
        let img = image::RgbImage::new(1920, 1080);
        let resized = resize_image(&img, 1920, 1080, [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(500, 1080);
        let resized = resize_image(&img, 1920, 1080, [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(1920, 500);
        let resized = resize_image(&img, 1920, 1080, [0, 0, 0]).unwrap();
        assert_eq!(resized.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(1920, 0);
        let resized = resize_image(&img, 1920, 1080, [0, 0, 0]);
        assert!(resized.is_err());

        let img = image::RgbImage::new(0, 1080);
        let resized = resize_image(&img, 1920, 1080, [0, 0, 0]);
        assert!(resized.is_err());
    }

    #[test]
    fn test_pad() {
        let img = image::RgbImage::new(1920, 1080);
        let padded = pad(&img, 1920, 1080, [0, 0, 0]).unwrap();
        assert_eq!(padded.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(500, 1080);
        let padded = pad(&img, 1920, 1080, [0, 0, 0]).unwrap();
        assert_eq!(padded.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(1920, 500);
        let padded = pad(&img, 1920, 1080, [0, 0, 0]).unwrap();
        assert_eq!(padded.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(1920, 0);
        let padded = pad(&img, 1920, 1080, [0, 0, 0]);
        assert!(padded.is_ok());

        let img = image::RgbImage::new(0, 1080);
        let padded = pad(&img, 1920, 1080, [0, 0, 0]);
        assert!(padded.is_ok());
    }

    #[test]
    fn test_crop_image() {
        let img = image::RgbImage::new(1920, 1080);
        let cropped = crop_image(&img, 1920, 1080).unwrap();
        assert_eq!(cropped.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(500, 1080);
        let cropped = crop_image(&img, 1920, 1080).unwrap();
        assert_eq!(cropped.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(1920, 500);
        let cropped = crop_image(&img, 1920, 1080).unwrap();
        assert_eq!(cropped.len(), 1920 * 1080 * 3);

        let img = image::RgbImage::new(1920, 0);
        let cropped = crop_image(&img, 1920, 1080);
        assert!(cropped.is_err());

        let img = image::RgbImage::new(0, 1080);
        let cropped = crop_image(&img, 1920, 1080);
        assert!(cropped.is_err());
    }
}
